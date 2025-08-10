#!/bin/bash

exchange=BINANCE
currency=USDT
pairlist=BTC
start_date=2024-01-01
months=18

./cli --exchange $exchange --currency $currency --pairlist $pairlist --start-date $start_date --months $months
