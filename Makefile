# Variables
IMAGE_NAME = montekkundan/rustapp
IMAGE_TAG = 1.0.0
CONTAINER_NAME = rustapp
DB_CONTAINER_NAME = db
DOCKER_COMPOSE_FILE = docker-compose.yml

# Targets
.PHONY: all build run stop clean logs db-shell db-view-tables

all: build run

build:
	@echo "Building Docker images..."
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

run:
	@echo "Starting services with Docker Compose..."
	docker-compose -f $(DOCKER_COMPOSE_FILE) up -d

stop:
	@echo "Stopping services..."
	docker-compose -f $(DOCKER_COMPOSE_FILE) down

clean: stop
	@echo "Removing Docker images and volumes..."
	-docker rmi $(IMAGE_NAME):$(IMAGE_TAG) || echo "No such image"
	@if docker ps -aq > nul 2>&1; then docker rm $$(docker ps -aq); else echo "No containers to remove"; fi
	@if docker images -q > nul 2>&1; then docker rmi $$(docker images -q); else echo "No images to remove"; fi
	@if docker volume ls -q > nul 2>&1; then docker volume rm $$(docker volume ls -q); else echo "No volumes to remove"; fi

logs:
	@echo "Displaying logs..."
	docker-compose -f $(DOCKER_COMPOSE_FILE) logs -f

db-shell:
	@echo "Connecting to the database shell..."
	docker exec -it $(DB_CONTAINER_NAME) psql -U postgres -d postgres

db-view-tables:
	@echo "Viewing tables in the database..."
	docker exec -it $(DB_CONTAINER_NAME) psql -U postgres -d postgres -c "\dt"
