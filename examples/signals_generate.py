import sys

from modules.database import Database
from modules.backtest_lab import BacktestLab

class SignalsGenerate:
    def __init__(self):
        self.database = Database()
        self.backtest_lab = BacktestLab(self.database)

    def _create_signal(self, symbol, action, stop_loss, take_profit, source, date):
        return {
            'symbol': symbol,
            'action': action,
            'SL': stop_loss,
            'TP': take_profit,
            'source': source,
            'date': date,
        }

    def generate_bollinger(self, symbol):
        TIME_FRAME = 20
        STD_DEV_FACTOR = 2.0

        prices = list(self.database.find_prices(symbol))
        self.backtest_lab.prices_add({symbol: prices})
        for i in range(5, len(prices)): # Skip first 5 prices
            mean = self.backtest_lab.algo_mean(symbol, 'bid', 'close', 0, i, TIME_FRAME)
            std_dev = self.backtest_lab.algo_std_dev(symbol, 'bid', 'close', 0, i, TIME_FRAME, mean)
            upper_band = mean + STD_DEV_FACTOR * std_dev
            lower_band = mean - STD_DEV_FACTOR * std_dev

            signal = None
            if prices[i]['close'] < lower_band:
                stop_loss = prices[i]['close'] - std_dev * STD_DEV_FACTOR / 2
                signal = self._create_signal(symbol, 'sell', stop_loss, [upper_band], 'Bollinger', prices[i]['date'])
            elif prices[i]['close'] > upper_band:
                stop_loss = prices[i]['close'] + std_dev * STD_DEV_FACTOR / 2
                signal = self._create_signal(symbol, 'buy', stop_loss, [lower_band], 'Bollinger', prices[i]['date'])
            
            if signal:
                self.database.insert_signal(signal)
                

if __name__ == '__main__':
    arguments = sys.argv
    symbol = arguments[1]
    SignalsGenerate().generate_bollinger(symbol)