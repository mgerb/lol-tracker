version=$(git describe --tags);
docker build -t ghcr.io/mgerb/lol-tracker:latest .;
docker tag ghcr.io/mgerb/lol-tracker:latest ghcr.io/mgerb/lol-tracker:$version;
