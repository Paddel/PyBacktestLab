# PyBacktestLab
A simple and efficient backtesting library for Python written in Rust.

### Output metrics:
| Metric | Description |
| ------ | ----------- |
| profit | Total profit |
| num_trades | Number of opened positions |
| hit_rate | Percentage of positions with positive result |
| sortino_ratio | Ratio of profit to risk |
| profit_per_day | Ratio of profit to number of days |
| positions | Info for all positions to calculate other metrics |

### Dependencies:
[Maturin](https://github.com/PyO3/maturin) is required to build the library.
Install with:
`pipx install maturin` or `pip install maturin`

To run the example scripts provided, run a MongoDB on `127.0.0.1:27017`.
Port and address can be adjusted in `examples/database.py`.

Also install the requirements:
`pip install -r examples/requirements.txt`

### Build:
Use `maturin build` to compile the library to a .whl file.
Use `pip install --force-reinstall path/to/whl_file.whl` to install the library.
Example: `pip install --force-reinstall target\wheels\PyBacktestLab-0.1.0-cp311-none-win_amd64.whl`

### Examples:
Run `python examples/prices_fetch.py EURUSD 2024 2` to fetch the EUR/USD price data for February 2024.

Run `python examples/signals_generate.py EURUSD` to generate signals with bollinger bands.

Run `python examples/signals_evaluate.py` to calculate the [output metrics](#Output_metrics) for the generated signals.