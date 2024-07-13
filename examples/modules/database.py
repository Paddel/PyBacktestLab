from pymongo import MongoClient

class Database:
    def __init__(self, host='localhost', port=27017, db_name='pybacktestlab'):
        self.db_name = db_name
        self.client = MongoClient(host, port)
        self._check_connection()

    def _check_connection(self):
        try:
            self.client.server_info()
        except Exception as e:
            raise Exception("Could not connect to database: " + str(e))
        
    def _get_db(self):
        return self.client[self.db_name]
    
    def _create_date_index(self, collection):
        try:
            collection.create_index('date', unique=True)
        except Exception as e:
            print(f"Could not create date index for collection {collection.name}: {str(e)}")

    def insert_signal(self, signal):
        self._get_db()['signals'].insert_one(signal)

    def insert_signal_batch(self, batch):
        self._get_db()['signals'].insert_many(batch, ordered=False)

    def update_signal(self, id, signal_update):
        self._get_db()['signals'].update_one({'_id': id}, {'$set': signal_update})

    def insert_asset_price_tick(self, symbol, date, bid, ask):
        symbol = "sym_" + symbol
        insert = {'date': date, 'bid': bid, 'ask': ask}
        collection = self._get_db()[symbol]
        collection.insert_one(insert)

    def insert_asset_price_tick_batch(self, symbol, batch):
        symbol = "sym_" + symbol
        collection = self._get_db()[symbol]
        try:
            collection.insert_many(batch, ordered=False)
        except:
            pass

    def insert_asset_price_ohlc(self, symbol, date, open, high, low, close):
        symbol = "sym_" + symbol
        insert = {'date': date, 'open': open, 'high': high, 'low': low, 'close': close}
        collection = self._get_db()[symbol]
        collection.insert_one(insert)

    def insert_asset_price_ohlc_batch(self, symbol, batch):
        symbol = "sym_" + symbol
        collection = self._get_db()[symbol]
        try:
            collection.insert_many(batch, ordered=False)
        except:
            pass

    def find_signals_by_source(self, source):
        return self._get_db()['signals'].find({'source': source})

    def find_signal_by_message_id(self, source, message_id):
        return self._get_db()['signals'].find_one({'source': source, 'message_id': message_id})
    
    def find_all_signals(self):
        return self._get_db()['signals'].find().sort("date", 1)
    
    def find_all_signals_after(self, date):
        return self._get_db()['signals'].find({'date': {'$gte': date}}).sort("date", 1)
    
    def find_all_signals_between(self, start_date, end_date):
        return self._get_db()['signals'].find({'date': {'$gte': start_date, '$lt': end_date}}).sort("date", 1)
    
    def find_prices(self, symbol):
        symbol = "sym_" + symbol
        return self._get_db()[symbol].find().sort("date", 1)
    
    def find_prices_after(self, symbol, date):
        symbol = "sym_" + symbol
        return self._get_db()[symbol].find({'date': {'$gte': date}}).sort("date", 1)
    
    def find_prices_between(self, symbol, start_date, end_date):
        symbol = "sym_" + symbol
        return self._get_db()[symbol].find({'date': {'$gte': start_date, '$lt': end_date}}).sort("date", 1)