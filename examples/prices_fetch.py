
from datetime import datetime
import os
import sys
import zipfile

from histdata import download_hist_data 
from histdata.api import Platform, TimeFrame

from modules.database import Database

TMP_PRICE_DIRECTORY = '.prices'
BATCH_SIZE = 10000

class CollectPricesHistorical:
    def __init__(self):
        self.database = Database()
        self.tmp_price_directory = TMP_PRICE_DIRECTORY

    def _collect_for_symbol_ohlc(self, year, month, symbol):
        download_hist_data(year=year, month=month, pair=symbol, platform=Platform.META_TRADER, time_frame=TimeFrame.ONE_MINUTE)
        file_base = f'DAT_MT_{symbol}_M1_{year}'
        if month:
            file_base += f'{month:02d}'
        with zipfile.ZipFile(file_base + '.zip', 'r') as myzip:
            with myzip.open(file_base + '.csv') as myfile:
                entries = {}
                for line in myfile:
                    line_parts = line.decode('utf-8').strip().split(',')
                    date = datetime.strptime(line_parts[0] + ' ' + line_parts[1], '%Y.%m.%d %H:%M')
                    entry = {
                        'date': date,
                        'open': float(line_parts[2]),
                        'high': float(line_parts[3]),
                        'low': float(line_parts[4]),
                        'close': float(line_parts[5]),
                    }
                    entries[date] = entry
                entires_sorted = dict(sorted(entries.items(), key=lambda x: x[0]))
                for date, entry in entires_sorted.items():
                    self.database.insert_asset_price_ohlc(symbol, entry['date'], entry['open'], entry['high'], entry['low'], entry['close'])	
        os.remove(file_base + '.zip')

    def _collect_for_symbol_tick(self, year, month, symbol):
        download_hist_data(year=year, month=month, pair=symbol, platform=Platform.GENERIC_ASCII, time_frame=TimeFrame.TICK_DATA)
        file_base = f'DAT_ASCII_{symbol}_T_{year}'
        if month:
            file_base += f'{month:02d}'
        with zipfile.ZipFile(file_base + '.zip', 'r') as myzip:
            with myzip.open(file_base + '.csv') as myfile:
                batch = []
                for line in myfile:
                    line_parts = line.decode('utf-8').strip().split(',')
                    date = datetime.strptime(line_parts[0], '%Y%m%d %H%M%S%f')
                    entry = {
                        'date': date,
                        'bid': float(line_parts[1]),
                        'ask': float(line_parts[2]),
                    }
                    batch.append(entry)
                    if len(batch) >= BATCH_SIZE:
                        self.database.insert_asset_price_tick_batch(symbol, batch)
                        batch = []
                if batch:
                    self.database.insert_asset_price_tick_batch(symbol, batch)
        os.remove(file_base + '.zip')

    def run(self, symbol, year, month):
        os.mkdir(self.tmp_price_directory)
        os.chdir(self.tmp_price_directory)
        self._collect_for_symbol_ohlc(year, month, symbol)
        os.chdir('..')
        os.removedirs(self.tmp_price_directory)
        

if __name__ == '__main__':
    arguments = sys.argv
    symbol = arguments[1]
    year = int(arguments[2])
    month = int(arguments[3])
    CollectPricesHistorical().run(symbol, year, month)