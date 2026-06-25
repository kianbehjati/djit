services:
  web:
    build: .
    ports:
      - 8000:8000
    volumes:
      - .:/code
    {{#if use_db}}
    depends_on:
      - db
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
  {{/if}}