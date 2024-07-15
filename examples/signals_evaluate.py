
from modules.database import Database
from modules.backtest_lab import BacktestLab

START_CAPITAL = 5000
COMMISSION = 10 # cents per lot
LOT_SIZE = 0.01
CONTRACT_SIZES = {'XAUUSD': 100}

class SignalsEvaluate:
    def __init__(self):
        self.database = Database()
        self.backtest_lab = BacktestLab(self.database)

    def _create_conditions(self):
        return {
            'max_margin': START_CAPITAL,
            'commission': COMMISSION,
            'lot_size': LOT_SIZE,
            'contract_sizes': CONTRACT_SIZES,
        }

    def evaluate(self):
        SOURCE = 'Bollinger'
        signals = list(self.database.find_all_signals())
        self.backtest_lab.prices_add_by_signals(signals)
        conditions = self._create_conditions()
        strategies = {SOURCE: {'entry': {'name': 'immediate', 'parameters': {}}, 'exit': {'name': 'fixed_tp', 'parameters': {'tp_factor': 1.0, 'vol_timeframe': 2.0}}, 'filter': {'name': 'no_filter', 'parameters': {}}}}
        result = self.backtest_lab.backtest_signals(conditions, strategies, signals)
        print(f"Profit: {result[SOURCE]['profit']}, Num Trades: {result[SOURCE]['num_trades']}, Hit Rate: {result[SOURCE]['hit_rate']}, Sortino Ratio: {result[SOURCE]['sortino_ratio']}")

if __name__ == '__main__':
    SignalsEvaluate().evaluate()