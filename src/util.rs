use anyhow::Result;
use url::Url;

static CURRENT_LOL_PATCH: &str = "13.21.1";

pub fn get_champion_image_url(champion_name: &str) -> Result<String> {
    let re = regex::Regex::new(r"[' ]")?;

    // remove space and single quotes from champion str
    let champion_name = re.replace_all(champion_name, "");

    let thumbnail_url = format!(
        "https://cdn.communitydragon.org/{}/champion/{}/square",
        CURRENT_LOL_PATCH, champion_name
    );

    Ok(Url::parse(&thumbnail_url)?.to_string())
}

pub fn get_author_url(summoner_name: &str) -> Result<String> {
    let author_url = Url::parse(&format!(
        "https://www.leagueofgraphs.com/summoner/na/{}",
        &summoner_name
    ))?
    .to_string();
    Ok(author_url)
}
