version=$(git describe --tags);
docker push ghcr.io/mgerb/lol-tracker:latest;
docker push ghcr.io/mgerb/lol-tracker:$version;
