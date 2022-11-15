import csv
import os


'''

###
Recorded parameters for evaluation
###

traffic_bytes: Bytes that were up- and downloaded between the proxy and the clients
ciphers_up: Amout of ciphers that were uploaded from the proxy to the clients
ciphers_down: Amount of ciphers that were downloaded from the clients to the proxy
cipher_size: Size of the public key, relinearization key and galois key that are used by the proxy
op_local: Average duration for an operation that was computed locally
levels: CKKS Level that specifies the depth of successive multiplications
op_offload: Duration of an offloaded operation (without latencies)
benchmarking_clients: Average benchmarking duration for one company
client_agg: Aggregation duration on the statistics server
keygen: Duration for key generation, which depends on the level and poly_modulus_size
keygen_size: Size of the public key, relinearization key and galois key that are used by the proxy
sample: Path of the utilized algorithm (contains the name)
benchmarking: Overall benchmarking duration
proxy_agg: Aggregation duration on the proxy
server_agg: Aggregation duration on the statistics server
accuracy: Average percentual deviation from the plaintext result value of all participants kpis
offloaded_pct: Percentage of operations that were required to be offloaded

'''


def init():
    global res
    res = {}
    res["traffic_bytes"] = 0
    res["ciphers_up"] = 0
    res["ciphers_down"] = 0
    res["cipher_size"] = {}
    res["op_local"] = []
    res["levels"] = 0
    res["op_offload"] = []
    res["benchmarking_clients"] = []
    res["client_agg"] = []
    res["keygen"] = 0
    res["keygen_size"] = 0


def write_eval(path):

    # Compute Operation info
    res["offloaded_pct"] = len(res["op_offload"]) / (len(res["op_local"]) + len(res["op_offload"]))

    # Compute averages
    res["op_local"] = sum(res["op_local"]) / len(res["op_local"]) if len(res["op_local"]) != 0 else -1
    res["op_offload_count"] = len(res["op_offload"])
    res["op_offload"] = sum(res["op_offload"]) / len(res["op_offload"]) if len(res["op_offload"]) != 0 else -1
    res["benchmarking_clients"] = sum(res["benchmarking_clients"]) / len(res["benchmarking_clients"]) if len(res["benchmarking_clients"]) != 0 else -1
    res["client_agg"] = sum(res["client_agg"]) / len(res["client_agg"]) if len(res["client_agg"]) != 0 else -1

    # Open CSV to write results into
    with open(path, 'a') as eval_file:

        writer = csv.DictWriter(eval_file, fieldnames=list(res.keys()))

        # Create header rows if required
        if os.stat(path).st_size == 0:
            writer.writeheader()

        writer.writerow(res)


def compute_accuracy(cleartext_clients, inaccurate_clients):

    errors = []

    for part in range(len(inaccurate_clients)):
        kpi_comp = inaccurate_clients[part].result.keys()
        in_client_res = inaccurate_clients[part].result
        ac_client_res = cleartext_clients[part].result

        # Absolute difference for HE and Cleartext
        for kpi in kpi_comp:
            if ac_client_res[kpi][0] != 0:

                abs_diff = abs(ac_client_res[kpi][0] - in_client_res[kpi][0])
                accuracy_loss = abs_diff / ac_client_res[kpi][0] * 100

                if accuracy_loss > 2.0:
                    print("[WARN]", kpi, "[AC]", ac_client_res[kpi], "[IN]", in_client_res[kpi],
                          "[ABS DIFF]", round(abs_diff, 4), "[Error]", round(accuracy_loss, 4))
                errors.append(round(accuracy_loss, 4))

    # Average error for all formulas and clients
    res["accuracy"] = sum(errors) / len(errors)

    # print("AVG Acc:", res["accuracy"])
