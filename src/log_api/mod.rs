use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Element, Html, Selector};

use crate::{
    api_strategy::ApiStrategy,
    dtos::{active_game_dto::ActiveGameDto, game_dto::GameDto, summoner_dto::SummonerDto},
};

// declare global const string user agent
static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36";

/// leagueofgraphs.com api

pub struct LogApiStrategy;

impl LogApiStrategy {
    fn get_selector(&self, selector_text: &str) -> Result<Selector> {
        let selector = match Selector::parse(selector_text) {
            Ok(s) => Ok(s),
            Err(_) => Err(anyhow::anyhow!(
                "Unable to parse selector: {}",
                selector_text
            )),
        }?;

        Ok(selector)
    }
}

#[async_trait]
impl ApiStrategy for LogApiStrategy {
    async fn get_summoner(&self, summoner_name: &str, guild_id: i64) -> Result<SummonerDto> {
        let url = format!(
            "https://www.leagueofgraphs.com/summoner/na/{}",
            summoner_name
        );

        let client = reqwest::Client::new();
        let body = client
            .get(url)
            .header("Cache-Control", "max-age=0")
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .context("get_summoner failed")?
            .text()
            .await
            .context("get_summoner failed to get text")?;

        let html = Html::parse_document(&body);
        let best_league_selector = self.get_selector(".best-league");
        let container = html
            .select(&best_league_selector?)
            .next()
            .context("unable to select .best-league")?;

        let selector = self.get_selector(".leagueTier")?;
        let val = container
            .select(&selector)
            .next()
            .context("unable to select .leagueTier")?;
        let league_tier = val.inner_html();
        let league_tier: Vec<&str> = league_tier.trim().split(" ").collect();
        let tier = league_tier.get(0).map(|s| s.to_string());
        let division = league_tier.get(1).map(|s| s.to_string());

        let selector = self.get_selector(".queueLine .queue")?;
        let queue_type = container
            .select(&selector)
            .next()
            .map(|val| val.inner_html().trim().to_string());

        let selector = self.get_selector(".league-points .leaguePoints")?;
        let lp = container
            .select(&selector)
            .next()
            .map(|val| val.inner_html().trim().to_string())
            .map_or(None, |val| val.parse::<i64>().ok());

        let summoner_img_selector = self.get_selector(".pageBanner .img img")?;

        let summoner_name_formatted = html
            .select(&summoner_img_selector)
            .next()
            .context("unable to select .pageBanner .img img")?
            .attr("title")
            .context("unable to get title")?;

        let icon_url = html
            .select(&summoner_img_selector)
            .next()
            .context("unable to select .pageBanner .img img")?
            .attr("src")
            .context("unable to get src")?
            .to_string();
        let icon_url = format!("https:{}", icon_url);

        Ok(SummonerDto {
            id: summoner_name_formatted.to_string(),
            name: summoner_name_formatted.to_string(),
            guild_id,
            created_at: None,
            updated_at: None,
            queue_type,
            lp,
            tier,
            division,
            icon_url,
        })
    }

    async fn get_games(&self, summoner_id: &str) -> Result<Vec<GameDto>> {
        let url = format!("https://www.leagueofgraphs.com/summoner/na/{}", summoner_id);

        let client = reqwest::Client::new();
        let body = client
            .get(url)
            .header("Cache-Control", "max-age=0")
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .context("get_games failed")?
            .text()
            .await
            .context("get_games failed to get text")?;

        let html = Html::parse_document(&body);

        let recent_games_table_selector =
            self.get_selector(".recentGamesBox .recentGamesTable tbody")?;
        let recent_games_table = html
            .select(&recent_games_table_selector)
            .next()
            .context("unable to select .recentGamesTable")?;

        let tr_selector = self.get_selector("tr")?;

        let champion_container_selector = self.get_selector(".championContainer img")?;
        let victory_defeat_text_selector = self.get_selector(".victoryDefeatText")?;
        let kills_selector = self.get_selector(".kda .kills")?;
        let assists_selector = self.get_selector(".kda .assists")?;
        let deaths_selector = self.get_selector(".kda .deaths")?;
        let game_mode_selector = self.get_selector(".gameMode")?;
        let lp_selector = self.get_selector(".gameMode .lpChange")?;
        let promotion_change_text_selector =
            self.get_selector(".lpChange .lpChangePromoteContainer.requireTooltip")?;
        let script_selector = self.get_selector("script")?;
        let id_selector = self.get_selector("td a")?;

        let mut games: Vec<GameDto> = vec![];

        for ele in recent_games_table.select(&tr_selector) {
            if let Some(val) = ele.select(&champion_container_selector).next() {
                let champion = val.attr("title").context("Unable to get champion title")?;
                let win = ele
                    .select(&victory_defeat_text_selector)
                    .next()
                    .context("Unable to get victoryDefeatText")?
                    .inner_html()
                    .contains("Victory");
                let kills: i64 = ele
                    .select(&kills_selector)
                    .next()
                    .context("Unable to get kills")?
                    .inner_html()
                    .parse()?;
                let assists: i64 = ele
                    .select(&assists_selector)
                    .next()
                    .context("Unable to get assists")?
                    .inner_html()
                    .parse()?;
                let deaths: i64 = ele
                    .select(&deaths_selector)
                    .next()
                    .context("Unable to get deaths")?
                    .inner_html()
                    .parse()?;

                let lp_element = ele.select(&lp_selector).next();
                let lp = lp_element
                    .map(|s| s.inner_html())
                    .map(|s| s.trim().to_string())
                    .map_or(None, |s| {
                        let s = s.split(" ").collect::<Vec<&str>>();
                        let s = s.get(0).map(|s| s.to_string()).map_or(None, |s| {
                            let s = if s.starts_with("+") {
                                s.strip_prefix("+")
                                    .map_or("".to_string(), |f| f.to_string())
                            } else {
                                s
                            };
                            return s.parse::<i64>().ok();
                        });

                        return s;
                    });
                let promotion_change_text = ele
                    .select(&promotion_change_text_selector)
                    .next()
                    .map_or(None, |s| s.attr("tooltip"));
                let game_mode = ele
                    .select(&game_mode_selector)
                    .next()
                    .context("Unable to get gameMode")?
                    .attr("tooltip")
                    .context("Unable to get gameMode tooltip")?
                    .to_string();

                // Skip with an error if the game is ranked and we can't get the .lpChange element
                if game_mode.to_lowercase().contains("ranked") && lp_element.is_none() {
                    return Err(anyhow::anyhow!(
                        "Unable to get lp from ranked game: {} - {}",
                        summoner_id,
                        champion
                    ));
                }

                let script = ele
                    .select(&script_selector)
                    .next()
                    .context("Unable to get script")?
                    .inner_html();

                let re = Regex::new(r#"new Date\((\d+)\)"#).context("Unable to create regex")?;

                let capture = re
                    .captures_iter(&script)
                    .next()
                    .context("Unable to get capture")?;
                let unix_date: i64 = (&capture[1]).parse()?;
                // Divide because this is in milliseconds
                let unix_date = unix_date / 1000;

                let id = ele
                    .select(&id_selector)
                    .next()
                    .context("Unable to get id")?
                    .attr("href")
                    .context("Unable to get id href")?
                    .to_string();

                games.push(GameDto {
                    id,
                    summoner_id: summoner_id.to_string(),
                    created_at: None,
                    updated_at: None,
                    game_created_at: unix_date,
                    assists,
                    deaths,
                    kills,
                    win,
                    notified: false,
                    champion_name: champion.to_string(),
                    game_mode,
                    lp_change: lp,
                    promotion_text: promotion_change_text.map(|s| s.to_string()),
                })
            }
        }

        Ok(games)
    }

    async fn get_active_game(
        &self,
        summoner_id: &str,
        summoner_name: &str,
    ) -> Result<Option<ActiveGameDto>> {
        let url = format!(
            "https://porofessor.gg/partial/live-partial/na/{}",
            summoner_name
        );

        let client = reqwest::Client::new();
        let body = client
            .get(url)
            .header("Cache-Control", "max-age=0")
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .context("get_active_game failed")?
            .text()
            .await
            .context("get_active_game failed to get text")?;

        let html = Html::parse_document(&body);
        let summoner_card_selector =
            self.get_selector(&format!(r#"div[data-summonername="{}"]"#, summoner_name))?;
        let summoner_card = html.select(&summoner_card_selector).next();

        // Return early if the summoner is not in a game
        if summoner_card.is_none() {
            return Ok(None);
        }

        let summoner_card = summoner_card.context("unable to select summoner card")?;

        let champion_selector = self.get_selector(".imgColumn-champion>div img")?;
        let champion = summoner_card
            .select(&champion_selector)
            .next()
            .context("unable to select champion")?
            .attr("alt")
            .context("unable to get champion alt")?
            .to_string();

        let role_selector = self.get_selector("div.currentRole>img")?;
        let role = summoner_card
            .select(&role_selector)
            .next()
            .context("unable to select current role")?
            .attr("alt")
            .context("unable to get current role alt")?
            .to_string();

        let game_id_selector = self.get_selector("#spectate_button")?;
        let game_id_link = html
            .select(&game_id_selector)
            .next()
            .context("unable to select game id link")?;
        let game_id = game_id_link
            .attr("data-spectate-gameid")
            .context("unable to get game id")?;
        let spectate_link = game_id_link
            .attr("data-spectate-link")
            .context("unable to get game spectate link")?
            .to_string();

        let game_mode_selector = self.get_selector(".site-content-header>h2")?;
        let game_mode = html
            .select(&game_mode_selector)
            .next()
            .context("unable to select game mode")?
            .text()
            .next()
            .context("unable to get game mode text")?
            .trim()
            .to_string();

        let game_created_at_selector = self.get_selector("[data-game-creation]")?;

        let game_created_at = html
            .select(&game_created_at_selector)
            .next()
            .context("unable to select game duration")?
            .attr("data-game-creation")
            .context("unable to get game creation")?
            .parse::<i64>()?
            / 1000;

        Ok(Some(ActiveGameDto {
            id: game_id.to_string(),
            game_mode,
            game_created_at,
            created_at: None,
            summoner_id: summoner_id.to_string(),
            champion,
            role,
            spectate_link,
            notified: false,
        }))
    }
}
