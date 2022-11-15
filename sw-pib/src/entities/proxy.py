import operator
import time
import sys
import gc

from seal import *

import util.fileoperations
import util.algorithms
import util.resolved_values
import util.constants
import util.variable
import util.evaluation as evaluation
import util.encrypted_variable
import util.atomic
import util.seal_util as seal_util


class PrivacyProxy:

    def __init__(self, config):
        self.config = config.config
        self.clients = {}
        self.algorithm = {}

    def check_client(self, client):
        if client not in self.clients:
            print("Unknown Client")

    def read_algorithm(self, filepath):
        self.algorithm = util.algorithms.Algorithm(filepath)

    def read_intervals(self, filepath):
        self.intervals = util.fileoperations.read_yaml(filepath)

    def register_client(self, client):
        self.clients[client] = {}

    def unregister_client(self, client):
        self.clients.pop(client)

    def request_inputs(self, client):
        self.check_client(client)

        if not self.config[util.constants.NETWORKING]:
            client.send_inputs(self)
            return

        # TODO: Not Implemented

    def receive_inputs(self, client, inputs, crypto):
        self.check_client(client)

        # The evaluator and the input are given
        self.clients[client]['inputs'] = inputs
        self.clients[client]['crypto'] = crypto

    # We have a strict ordering of what is to be computed
    # as the next operation. With this premise we only need
    # to check if the next operation is ok or not. Otherwise
    # We do the offloading.

    def compute_kpis(self, client):

        # print("Offloading:", self.config["offload"])

        # This verifies that the input is not encrypted by the client
        # In case the client did not provide crypto attributes we infer plaintext
        crypto = None
        if self.clients[client]["crypto"] is not None:
            crypto = self.clients[client]["crypto"]

        # Required variables are initially "resolved"
        resolved_vals = util.resolved_values.ResolvedValues()

        # Required variables get the crypto context in case of cipher to be computed
        if crypto is None:
            for required_var in self.algorithm.required:
                company_input = self.clients[client]["inputs"][required_var]
                var_values = util.variable.Variable(company_input["values"])
                resolved_vals.insert(required_var, var_values)
        else:
            for required_var in self.algorithm.required:
                company_input = self.clients[client]["inputs"][required_var]
                var_values = util.encrypted_variable.EncVariable(
                    company_input["cipher"],
                    self.clients[client]["crypto"],
                    company_input["len"],
                )
                resolved_vals.insert(required_var, var_values)

        # Clear client first as data is no longer required now
        del self.clients[client]["inputs"]
        gc.collect()

        to_op = self.algorithm.operations.copy()
        deletion_counter = 1

        print("[INFO] Proxy will offload: ", self.config["offload"])

        # Compute every atomic in given order
        #
        # Computation will NOT fail because of missing
        # values since we verified them in advance.
        # Still there might be runtime errors e.g.
        # unused constants.. that get reported!
        for atom in self.algorithm.operations:
            # Compute atomic operation
            atomic = {
                "name": atom[1]["name"],
                "op": atom[1]["op"],
                "var": atom[1]["var"],
                "constant": atom[1]["constant"]
            }

            # Missing operations to do
            to_op.remove(atom)

            # Determine Operation and operands
            # Depending on whether we got crypto information we do compuation accordingly
            operands = util.atomic.get_resolved_for_op(atomic, resolved_vals, crypto)
            oper_input = util.atomic.OperationInput(operands)
            operation = util.atomic.get_op(atomic["op"])

            # Computeation result
            res = None

            # (A) Proxy: Local Computation:
            # Proxy: Compute Atomic Function
            # If you can compute the operation do it
            # Limitation of SEAL forces us to offload multiplications with dimension > 1
            if atomic["op"] not in self.config["offload"] and not (atomic["op"] == "Multiplication" and max(map(lambda x: len(x), operands)) > 1):
                # print("LOCAL COMPUTATION [", atomic["name"], "]:", atomic["op"], atomic["var"])

                # This operation might fail if the multiplicative depth is
                # exceeded. Thus we use fallback of sending it to the client
                # who performs the operation and returns an encrypted variable.
                try:

                    op_start_time = time.time()

                    res = operation(oper_input)

                    # print("LOCAL COMPUTATION OK!!")

                    # If operation went through → measure time
                    if self.config["evaluation"]:
                        if "op_local" in evaluation.res.keys():
                            evaluation.res["op_local"].append(time.time() - op_start_time)
                        else:
                            evaluation.res["op_local"] = [time.time() - op_start_time]

                except Exception as e:

                    print("[CRIT] UNEXPECTED OFFLOAD [::] →", atomic["op"], atomic["var"], "← DUE TO TOO LOW LEVEL!")
                    print("[CRIT] REASON", e)
                    print("[CRIT] GOING FURTHER...")

                    if self.config["mode"] == "encrypted":
                        evaluation.res["traffic_bytes"] += sum(map(lambda x: x.ciphertext.save_size(), oper_input.values))

                        # Upload and download counting
                        # Result is always one cipher thus we dont have to account for the rest
                        evaluation.res["ciphers_up"] += len(oper_input.values)
                        evaluation.res["ciphers_down"] += 1

                        # Measure upload and download size for one cipher and use it for subsequent multiplications
                        for i in oper_input.values:

                            if i.ciphertext.save_size() not in evaluation.res["cipher_size"].keys():
                                evaluation.res["cipher_size"][i.ciphertext.save_size()] = 1
                            else:
                                evaluation.res["cipher_size"][i.ciphertext.save_size()] += 1

                        if "op_offload" in evaluation.res.keys():
                            evaluation.res["op_offload"].append(time.time() - op_start_time)
                        else:
                            evaluation.res["op_offload"] = [time.time() - op_start_time]

                    res = client.compute_offloaded_operation(oper_input, operation)

            # (B) Proxy: Offloaded Computation:
            # Otherwise offload it to the participant who then decrypts it
            else:

                if self.config["mode"] == "encrypted":
                    evaluation.res["traffic_bytes"] += sum(map(lambda x: x.ciphertext.save_size(), oper_input.values))

                    # Upload and download counting
                    evaluation.res["ciphers_up"] += len(oper_input.values)
                    evaluation.res["ciphers_down"] += 1

                    # Measure upload and download size for one cipher and use it for subsequent multiplications
                    # if evaluation.res["cipher_size"] == -1:
                    #     evaluation.res["cipher_size"] = oper_input.values[0].ciphertext.save_size()
                    for i in oper_input.values:
                        if i.ciphertext.save_size() not in evaluation.res["cipher_size"].keys():
                            evaluation.res["cipher_size"][i.ciphertext.save_size()] = 1
                        else:
                            evaluation.res["cipher_size"][i.ciphertext.save_size()] += 1

                # print("OFFLOADING: ", atomic["op"])
                op_start_time = time.time()

                res = client.compute_offloaded_operation(oper_input, operation)

                # If operation went through → measure time
                if self.config["evaluation"]:
                    if "op_offload" in evaluation.res.keys():
                        evaluation.res["op_offload"].append(time.time() - op_start_time)
                    else:
                        evaluation.res["op_offload"] = [time.time() - op_start_time]

            # Insert result into resolved vars
            resolved_vals.insert(atomic["name"], res)

            # Do only on encryption mode and with
            if self.config["mode"] == "encrypted" and deletion_counter % 100 == 0:
                # Remove the inputs if they are no longer necessary to save memory (time penalty!)
                var_req_list = []
                for op in to_op:
                    var_req_list += op[1]["var"]

                todel = [x for x in resolved_vals.resolved if (x not in var_req_list) and (x in self.algorithm.non_kpis)]
                resolved_vals.filter_atomics_by_name(todel)

                del(todel)
                del(var_req_list)

                gc.collect()

            # Increase counter for higher (constant) performance win and higher (constant) performance usage
            deletion_counter += 1

        # Filter non_kpis
        resolved_vals.filter_atomics_by_name(self.algorithm.non_kpis)
        resolved_vals.filter_atomics_by_name(self.algorithm.required)

        # Insert the currated result list for the client to his results
        self.clients[client]['results'] = resolved_vals

    # Compute intermediary aggregates for statistics server
    # For now: no minimum and maximum given → either use Order preserving encryption
    #                                       → or the other paper that implemented it
    #                                         with multiplication. However offloading
    #                                         is not possible in this scenario (afaik).
    def compute_intermediary_aggregates(self, client_inputs, statistics_server_crypto):

        kpi_clusters = {}

        for client_kpis in client_inputs:
            # Verify that all KPIs were really provided by the clients and agg. only those
            for kpi in self.algorithm.kpis:
                if client_kpis[kpi["name"]] is None:
                    raise Exception("Client did not provide KPI", kpi)

                ciphertext = client_kpis[kpi["name"]]["cipher"]
                length = client_kpis[kpi["name"]]["len"]

                # Use an encrypted variable for this purpose
                enc_var = util.encrypted_variable.EncVariable(ciphertext, statistics_server_crypto, length)

                # Create entries on first user
                if kpi["name"] not in kpi_clusters.keys():
                    kpi_clusters[kpi["name"]] = {
                        "len": length,
                        "kpis": [enc_var]
                    }
                else:
                    if kpi_clusters[kpi["name"]]["len"] != length:
                        raise Exception("Dimension missmatch - two companies provided KPIs with different dimensions - expected",
                                        kpi_clusters[kpi["name"]]["len"], ", given", length, "!")
                    # Insert an encrypted variable into the  existing list
                    kpi_clusters[kpi["name"]]["kpis"].append(enc_var)

        kpi_aggregates = {}

        # Now aggregate the computed clusters
        for cluster in kpi_clusters.items():
            # Put all ciphertexts inside the input
            agg_input = util.atomic.OperationInput(cluster[1]["kpis"])
            agg_op = util.atomic.get_op("Addition")
            res = agg_op(agg_input)

            kpi_aggregates[cluster[0]] = {
                "min": 0,                           # Minimum. Cannot compute for now
                "max": 0,                           # Maximum. Cannot compute for now
                "sum": res.ciphertext,              # Sum over all inputs
                "len": len(cluster[1]["kpis"][0]),  # Length of one KPI
                "comp": len(cluster[1]["kpis"]),    # Number of participants
            }

            # print(kpi_aggregates[cluster[0]])

        # With the aggregates → send to statistics server who then does the rest
        return kpi_aggregates
