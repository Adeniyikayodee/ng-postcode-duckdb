# ng_postcode for DuckDB

A DuckDB extension for Nigeria's NIPOST digital postcode (NDAPS), the 11-character building code such as `EK-01-A03-FK-01`. It parses and formats codes, and cuts them down to a state, LGA, district, or area, so that a table or a CSV file can be grouped by place with no map.

The work is done in Rust by the same core as the [`ng-postcode`](https://github.com/Adeniyikayodee/ng-postcode) libraries, and the functions have the names and answers of that project's [SQL macros](https://github.com/Adeniyikayodee/ng-postcode/tree/main/sql). On a million rows the extension parsed, truncated, and formatted every code in 0.06 seconds on one laptop, against 0.21 seconds for the macros.

## Use

Needs DuckDB 1.5.6; see [DuckDB versions](#duckdb-versions).

```sql
INSTALL ng_postcode FROM community;
LOAD ng_postcode;

SELECT ng_postcode_parse('ek 01 a03 fk 01');
-- EK01A03FK01

SELECT ng_postcode_format(ng_postcode_truncate(postcode, 'district')) AS district, count(*)
FROM read_csv('deliveries.csv')
GROUP BY 1;
```

A value that is not a well-formed code gives NULL, so it can be counted and followed up. To keep such values out of a table, add a check to the column:

```sql
CREATE TABLE customers (
    id       bigint PRIMARY KEY,
    postcode text CHECK (ng_postcode_parse(postcode) IS NOT DISTINCT FROM postcode)
);
```

Write the check with `IS NOT DISTINCT FROM`. With a plain `=`, a malformed code is compared with NULL, and a check lets a row through unless the answer is false.

## Functions

| Function | Returns |
| --- | --- |
| `ng_postcode_parse(text)` | The compact code, `EK01A03FK01`, or NULL |
| `ng_postcode_prefix_parse(text)` | The compact prefix, such as `EK01A03`, or NULL |
| `ng_postcode_format(code or prefix)` | The hyphenated form |
| `ng_postcode_truncate(code, segment)` | The prefix through `state`, `lga`, `district`, `area`, or `unit` |
| `ng_postcode_parent(prefix)` | The prefix one segment shorter, or NULL for a state |
| `ng_postcode_within(code, prefix)` | Whether the code starts with the prefix |
| `ng_postcode_compact(text)` | The letters and digits in upper case, or NULL if the text holds anything else |
| `ng_postcode_version()` | The version of the extension |

A well-formed code is not necessarily assigned to a building. Only the NIPOST API can confirm that, through one of the `ng-postcode` libraries.

## DuckDB versions

The community repository carries the extension for DuckDB 1.5.6. DuckDB's Rust support ties a build to one release, so on any other version `INSTALL` reports that the extension is not found.

## Build and test

Clone with submodules, then build and run the tests. You need Rust, Python 3, and Make.

```sh
git clone --recurse-submodules https://github.com/Adeniyikayodee/ng-postcode-duckdb
cd ng-postcode-duckdb
make configure
make release
make test_release
```

A build works with one version of DuckDB, named in the `Makefile`, because DuckDB's Rust support still uses an interface that changes between releases. To try a local build, start DuckDB with `-unsigned` and load the file by its full path:

```sql
LOAD '/full/path/to/build/release/ng_postcode.duckdb_extension';
```

## Tests

`test/sql/vectors.test` is written by `scripts/vectors_test.py` from `test/vectors.json`, a copy of the cases every `ng-postcode` implementation must pass. `test/sql/ng_postcode.test` covers NULLs, columns, and grouping.

## License

MIT
