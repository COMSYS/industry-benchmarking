# Accepts SIGINT and thus can be used to attatch to it
docker run -it --device=/dev/isgx -p 8080:8080 -p 8443:8443 -d --hostname=teebench.xyz --name server_encrypted --rm teebench_server_image:latest
