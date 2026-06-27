services:
  web:
    build: .
    ports:
      - 8000:8000
    volumes:
      - .:/code
    {{#if use_db}}
    depends_on:
      db:
        condition: service_healthy
        restart: true
    {{/if}}
    command: python manage.py runserver 0.0.0.0:8000
  {{#if use_db}}
  db:
    image: {{use_db}}:{{db_tag}}
    volumes:
      {{#if is_postgres}}
      - ./data/db:/var/lib/postgresql
      {{else}}
      - ./data/db:/var/lib/mysql
      {{/if}}
    environment:
      {{#if is_postgres}}
      - POSTGRES_DB={{db_option.db_name}}
      - POSTGRES_USER={{db_option.db_user}}
      - POSTGRES_PASSWORD={{db_option.db_password}}
      {{else}}
      - MYSQL_DATABASE={{db_option.db_name}}
      - MYSQL_ROOT_PASSWORD={{db_option.db_password}}
      {{/if}}
    {{#if is_postgres}}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $${POSTGRES_USER} -d $${POSTGRES_DB}"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    {{else}}
    healthcheck:
      test: ["CMD-SHELL", "mysqladmin ping -h localhost -u$${MYSQL_USER} -p$${MYSQL_PASSWORD} --silent"]
      interval: 10s
      timeout: 10s
      retries: 5
      start_period: 30s
    {{/if}}
  {{/if}}