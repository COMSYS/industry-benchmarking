import psutil
import gc
import argparse
import os
import time
import threading

import entities.client
import entities.proxy
import entities.server
import util.evaluation as evaluation
import util.configurations
from util.variable import *
from util.atomic import *


# Testing example
algorithm_file = "../data/Test/Algorithms/atomics.yaml"
inputs_dir = "../data/Test/Inputs/"

# Output directories for evaluation results
eval_file = "../data/Results/results.csv"

# Configuartion information
settings_file = "./settings/PIB_SEAL.yaml"

# Clients
clients = []
THREADING = True


def do_accuracy(config, parsed_args):
    config.config["evaluation"] = False
    config.config["mode"] = "plaintext"

    # Restart the proxy with the new config
    proxy = entities.proxy.PrivacyProxy(config)
    proxy.read_algorithm(parsed_args.algorithms)

    clients = []
    server = entities.server.StatisticsServer(config)
    # Client: Read Input Data
    # Compute for each client indivdually
    for company_input in util.fileoperations.get_all_yaml_files_in_dir(parsed_args.inputs):

        # Create client from configuration and with the input file
        client = entities.client.Company(config, company_input, proxy.algorithm.required)

        # Register client
        proxy.register_client(client)

        # Upload the requested input
        proxy.request_inputs(client)

        # Global: Check Intervals of Input Data
        # check_intervals(proxy.intervals, client.inputs)

        # Compute the KPIs for each client individually
        proxy.compute_kpis(client)

        # Client gets result and decrypts it
        client.get_results(proxy.clients[client]["results"].get_resolved())

        # keep client for later aggregation
        clients.append(client)

        print("[SUCC] Cleartext Client Done")

    return clients


# Threading for faster evaluation
def compute_company(proxy, company_input, config, index):
    start_time_client = time.time()

    # Create client from configuration and with the input file
    client = entities.client.Company(config, company_input, proxy.algorithm.required)

    # Register client
    proxy.register_client(client)

    # Upload the requested input
    proxy.request_inputs(client)

    # Global: Check Intervals of Input Data
    # check_intervals(proxy.intervals, client.inputs)

    # Compute the KPIs for each client individually
    proxy.compute_kpis(client)

    # Client gets result and decrypts it
    client.get_results(proxy.clients[client]["results"].get_resolved())

    # Measure time for evaluation to know how long one company took
    if config.config["evaluation"]:
        exec_time = (time.time() - start_time_client)
        if "benchmarking_clients" in evaluation.res.keys():
            evaluation.res["benchmarking_clients"].append(exec_time)
        else:
            evaluation.res["benchmarking_clients"] = [exec_time]

    print("[INFO] Client Done")

    clients[index] = client


if __name__ == '__main__':

    print("### SW-PIB ###")

    # Global: Parse Commandline Arguments
    parser = argparse.ArgumentParser("PIB Execution")
    parser.add_argument('-a', '--algorithms', type=str, dest='algorithms',
                        default=algorithm_file, help='Atomic algorithm file\n DEFAULT: ' + algorithm_file)
    parser.add_argument('-i', '--inputs', type=str, dest='inputs', default=inputs_dir,
                        help='Directory where the "comp00.yaml" ... "compN.yaml" are inputs.\n DEFAULT: ' + inputs_dir)
    parser.add_argument('-e', '--eval', type=str, dest='eval', default=eval_file,
                        help='File where the eval is written to.\n DEFAULT: ' + eval_file)
    parser.add_argument('-c', '--config', type=str, dest='config', default=settings_file,
                        help='Configuration file for the Proxy (relevant or eval)\n DEFAULT: ' + settings_file)

    parsed_args = parser.parse_args()

    print("[INFO] Setting up key material for fully homomorphic encryption...")

    # Global: Parse Eval Configuration File
    config = util.configurations.Config(parsed_args.config)

    evaluation.init()

    evaluation.res["sample"] = parsed_args.algorithms

    # Globally create and configure entities

    # Proxy
    proxy = entities.proxy.PrivacyProxy(config)
    proxy.read_algorithm(parsed_args.algorithms)

    server = entities.server.StatisticsServer(config)

    files = util.fileoperations.get_all_yaml_files_in_dir(parsed_args.inputs)
    threads = [None] * len(files)
    clients = [None] * len(files)

    print("[INFO] Starting Benchmarking process...\n")

    all_start_time = time.time()

    # Client: Read Input Data
    # Compute for each client indivdually
    for company_input in zip(files, list(range(len(files)))):
        # Register and start threads
        # Only if configured
        if THREADING:
            threads[company_input[1]] = threading.Thread(target=compute_company, args=(proxy, company_input[0], config, company_input[1]))
            threads[company_input[1]].start()
        else:
            compute_company(proxy, company_input[0], config, company_input[1])

    if THREADING:
        for i in range(len(files)):
            # keep client for later aggregation
            threads[i].join()

    # Measure time for evaluation to know how long everything took
    if config.config["evaluation"]:
        exec_time = (time.time() - all_start_time)
        evaluation.res["benchmarking"] = exec_time

    print("[SUCC] Benchmarking Complete")

    print("[INFO] Memory Usage:", psutil.Process().memory_info().rss / (1000 * 1000), "MB\n")

    # Aggregation phase
    # All clients send their results encrypted with the statistics server's public key
    # The proxy does aggregation and the proxy evaluates the statistics
    # The number k is implicitly given by the amount of inputs we have

    client_aggregation_inputs = []

    for client in clients:
        agg_time_client_start = time.time()

        client_aggregate = client.prepare_aggregation(server.provide_crypto())
        client_aggregation_inputs.append(client_aggregate)

        # Measure time for evaluation to know how long everything took
        if config.config["evaluation"]:
            exec_time = (time.time() - agg_time_client_start)
            if "client_agg" in evaluation.res.keys():
                evaluation.res["client_agg"].append(exec_time)
            else:
                evaluation.res["client_agg"] = [exec_time]

    agg_time_proxy = time.time()

    if len(clients) > 1:
        aggregation_intermediates = proxy.compute_intermediary_aggregates(client_aggregation_inputs, server.provide_crypto())

        # Measure time for evaluation to know how long the proxy took aggregating
        if config.config["evaluation"]:
            exec_time = (time.time() - agg_time_proxy)
            evaluation.res["proxy_agg"] = exec_time

        agg_time_server = time.time()

        print("[INFO] Computing Aggregation...")

        aggregation_result = server.compute_statistics(aggregation_intermediates, len(clients))

        # Measure time for evaluation to know how long the server took decryting and checking
        if config.config["evaluation"]:
            exec_time = (time.time() - agg_time_server)
            evaluation.res["server_agg"] = exec_time

        print("[SUCC] Aggregation Done\n")

        for client in clients:
            client.get_aggregation_result(aggregation_result)
            del client.inputs
            del client.aggregation_results

        # Free memory
        del aggregation_result

    else:
        for client in clients:
            del client.inputs
            del client.aggregation_results
        print("[WARN] Only one participant! Skipping aggregation..")

    print("[INFO] Cleaning up FHE objects...")
    # Free memory
    del proxy
    gc.collect()

    # Inaccurate computation
    inaccurate_clients = clients

    print("[INFO] Restarting compuation on plaintext for accuracy comparision...\n")

    # Restart Proxy
    proxy = entities.proxy.PrivacyProxy(config)
    proxy.read_algorithm(parsed_args.algorithms)

    # Compute accuracy → for this we need to recompute the entire benchmarking on unencrypted text
    # → set evaluation to false to not measure again and set the mode to plaintext
    cleartext_clients = do_accuracy(config, parsed_args)
    evaluation.compute_accuracy(cleartext_clients, inaccurate_clients)

    ###
    # Write Evaluation
    ###

    config.config["evaluation"] = True

    if config.config["evaluation"]:
        evaluation.write_eval(parsed_args.eval)

    print("### SW-PIB FINISHED ###")
