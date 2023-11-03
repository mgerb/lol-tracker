version=$(git describe --tags);
docker build -t mgerb/lol-tracker:latest .;
docker tag mgerb/lol-tracker:latest mgerb/lol-tracker:$version;
