from django.core.management.utils import get_random_secret_key
from sys import argv
with open('.env', 'w') as f:
    if len(argv) < 2:
        f.write(f'SECRET_KEY={get_random_secret_key()}\nDEBUG=True')
    elif len(argv) == 2:
        if len(argv[1]) > 0:
            f.write(f'SECRET_KEY={get_random_secret_key()}\nDEBUG=True\nDB_PASSWORD={argv[1]}')
        else:
            f.write(f'SECRET_KEY={get_random_secret_key()}\nDEBUG=True')
f.close()