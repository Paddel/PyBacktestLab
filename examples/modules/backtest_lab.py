
from datetime import timedelta

import py_backtest_lab

class BacktestLab:
    def __init__(self, database):
        self.database = database

    def _convert_rust_price_ohlc(price):
        return {
            'time_stamp': int(price['date'].timestamp() * 1000), # Milliseconds since epoch
            'open': price['open'],
            'high': price['high'],
            'low': price['low'],
            'close': price['close'],
        }
    
    def _convert_rust_prices_ohlc(prices):
        return [BacktestLab._convert_rust_price_ohlc(price) for price in prices]

    def _convert_rust_price_tick(price):
        return {
            'time_stamp': int(price['date'].timestamp() * 1000), # Milliseconds since epoch
            'bid': price['bid'],
            'ask': price['ask'],
            'sentiment': price['sentiment'],
        }

    def _convert_rust_prices_tick(prices):
        return [BacktestLab._convert_rust_price_tick(price) for price in prices]
    
    def _convert_rust_signal(signal):
        return {
            'symbol': signal['symbol'],
            'action': signal['action'],
            'stop_loss': signal['SL'],
            'take_profit': signal['TP'],
            'time_stamp': int(signal['date'].timestamp() * 1000), # Milliseconds since epoch
            'source': signal['source'],
        }
    
    def _convert_rust_signals(signals):
        return [BacktestLab._convert_rust_signal(signal) for signal in signals]

    def _signals_by_symbol(self, signals):
        signals_by_symbol = {}
        for signal in signals:
            symbol = signal['symbol']
            if symbol not in signals_by_symbol:
                signals_by_symbol[symbol] = []
            signals_by_symbol[symbol].append(signal)
        return signals_by_symbol
    
    def _filter_signals(self, signals):
        return [signal for signal in signals if all(isinstance(element, float) for element in signal['TP'])]
    
    def _assign_sentiemnts_to_prices(self, prices, sentiments):
        current_sentiment_index = 0
        sentiment = 50.0
        for price in prices:
            while current_sentiment_index < len(sentiments) - 2 and sentiments[current_sentiment_index + 1]['date'] <= price['date']:
                current_sentiment_index += 1
                sentiment = sentiments[current_sentiment_index]['sentiment']
            price['sentiment'] = sentiment
        return prices

    def prices_add_by_signals(self, signals, caching=True):
        signals_by_symbol = self._signals_by_symbol(signals)
        prices_by_symbol = {}
        for symbol, signals_for_symbol in signals_by_symbol.items():
            day_from = signals_for_symbol[0]['date'].replace(hour=0, minute=0, second=0, microsecond=0) - timedelta(days=1)
            day_to = signals_for_symbol[-1]['date'].replace(hour=0, minute=0, second=0, microsecond=0) + timedelta(days=1)
            prices = self.database.find_prices_between(symbol, day_from, day_to)
            prices_by_symbol[symbol] = []
            for price in prices:
                prices_by_symbol[symbol].append(price)
                if caching and len(prices_by_symbol[symbol]) >= 100000:
                    self.prices_add({symbol: prices_by_symbol[symbol]})
                    prices_by_symbol[symbol] = []
            if symbol in prices_by_symbol:
                self.prices_add({symbol: prices_by_symbol[symbol]})
        return prices_by_symbol

    def prices_add(self, prices):
        prices_by_symbol = {}
        type_tick = True if 'ask' in list(prices.values())[0][0] else False
        for symbol, symbol_prices in prices.items():
            if not symbol_prices:
                continue
            if type_tick:
                prices_by_symbol[symbol] = BacktestLab._convert_rust_prices_tick(symbol_prices)
            else:
                prices_by_symbol[symbol] = BacktestLab._convert_rust_prices_ohlc(symbol_prices)
        if type_tick:
            py_backtest_lab.prices_tick_add(prices_by_symbol)
        else:
            py_backtest_lab.prices_ohlc_add(prices_by_symbol)

    def backtest_signals(self, conditions, strategy_rules, signals):
        signals = self._filter_signals(signals)
        signals = BacktestLab._convert_rust_signals(signals)
        return py_backtest_lab.backtest_signals(conditions, strategy_rules, signals)
    
    def signal_check_filter(self, strategy_rules, signal):
        signal_rust = BacktestLab._convert_rust_signal(signal)
        return py_backtest_lab.signal_check_filter(strategy_rules, signal_rust)

    def signal_check_entry(self, strategy_rules, signal_result):
        return py_backtest_lab.signal_check_entry(strategy_rules, signal_result)
    
    def signal_check_exit(self, strategy_rules, signal_result):
        return py_backtest_lab.signal_check_exit(strategy_rules, signal_result)
    
    def algo_mean(self, symbol, type_tick, type_ohlc, index_from, index_to, minutes):
        return py_backtest_lab.algo_mean(symbol, type_tick, type_ohlc, index_from, index_to, minutes)
    
    def algo_std_dev(self, symbol, type_tick, type_ohlc, index_from, index_to, minutes, mean):
        return py_backtest_lab.algo_std_dev(symbol, type_tick, type_ohlc, index_from, index_to, minutes, mean)