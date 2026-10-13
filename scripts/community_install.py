"""Install ng_postcode from DuckDB's community repository and run the shared cases through it.

    pip install duckdb
    python3 scripts/community_install.py
"""

import sys

import duckdb

from vectors_test import checks

db = duckdb.connect()
db.execute("INSTALL ng_postcode FROM community")
db.execute("LOAD ng_postcode")
failed = [check for check in checks() if db.execute(f"SELECT {check}").fetchone()[0] is not True]
version = db.execute("SELECT ng_postcode_version()").fetchone()[0]
print(f"DuckDB {duckdb.__version__}, ng_postcode {version}: {len(failed)} failed")
if failed:
    sys.exit("\n".join(failed))
