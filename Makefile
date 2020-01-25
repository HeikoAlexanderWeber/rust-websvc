.PHONY: certs docker-build

certs:
	openssl req -new -newkey rsa:4096 -days 365 -nodes -x509 -keyout ./res/certs/server.key -out ./res/certs/server.crt

docker-build:
	docker build \
		-f docker/Dockerfile \
		-t rust-websvc:latest \
		. --rm
