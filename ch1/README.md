docker build -t rcore .
docker run -it --rm -v $(pwd):/mnt -w /mnt rcore
