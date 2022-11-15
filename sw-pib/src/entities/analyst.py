import argparse
import sys
import yaml
import json
import sqlite3
from flask import Flask, json
import requests
import os
import util.fileoperations as fs


'''
Read atomic KPI formulas
'''


def read_algorithms(path):
    return fs.read_yaml(path)


# For manually executing the analyst
if __name__ == "__main__":

    # Parse input from CLI
    parser = argparse.ArgumentParser('FHEBench Analyst')
    parser.add_argument('-a', '--algorithm', type=str,
                        help="Please provide the path to the algorithms")
    parsed_args = parser.parse_args()

    # Check that input is existing
    if parsed_args.algorithm:
        if not os.path.isfile(parsed_args.algorithm):
            raise TypeError("Algorithm file was not found!")
        else:
            path = parsed_args.algorithm

    algorithms = read_algorithms(path)
    kpi_names = list(map(lambda obj: obj['name'], algorithms['operations']))
