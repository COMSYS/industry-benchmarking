import util.fileoperations
import util.constants
from seal import *
import time
import sys
import util.seal_util as seal_util
import util.evaluation as evaluation


class Company:

    def __init__(self, config, filepath, required_vars):
        self.inputs = None
        self.config = config.config
        self.result = None
        self.aggregation_results = None

        # Input retrieval
        self.read_inputs(filepath)
        # Verify correct inputs
        self.verify_inputs(required_vars)

        # Encrypt if required
        if self.config["mode"] == "plaintext":
            self.crypto = None
        elif self.config["mode"] == "encrypted":

            start_time = time.time()

            print("[INFO] Selected crypto configuration: ", self.config["crypto"])

            self.crypto = self.crypto_gen(self.config["crypto"]["polymod"], self.config["crypto"]["level"], self.config["crypto"]["scale"])

            if config.config["evaluation"]:
                exec_time = (time.time() - start_time)
                evaluation.res["keygen"] = exec_time
                evaluation.res["keygen_size"] = sys.getsizeof(self.crypto["pub_key"].to_string())
                + sys.getsizeof(self.crypto["galois_keys"].to_string())
                + sys.getsizeof(self.crypto["relin_keys"].to_string())

            self.encrypt_inputs()
        else:
            raise Exception("Client: unrecognized mode:", self.config["mode"])

    def read_inputs(self, filepath):
        self.inputs = {}
        input_vars = util.fileoperations.read_yaml(filepath)

        for input in input_vars["vars"]:
            self.inputs[input["name"]] = input

    def verify_inputs(self, required_vars):
        for req_input in required_vars:
            if self.inputs.get(req_input) is None:
                raise Exception("Variable ", req_input, "was not found in the provided input")

    def encrypt_inputs(self):
        enc_inputs = {}
        for i in self.inputs.items():
            # Pad the missing slots
            # data = i[1]["values"] + [0] * (self.crypto["slots"] - len(i[1]["values"]))
            # print(i[0], "vec:")
            # seal_util.print_vector(data)

            input_plain = self.crypto["encoder"].encode(i[1]["values"], self.crypto["scale"])
            enc_inputs[i[0]] = {
                "cipher": self.crypto["encryptor"].encrypt(input_plain),
                "len": len(i[1]["values"])
            }
        self.inputs = enc_inputs

    def get_results(self, resolved_variables):
        # In case of encrypted results → decrypt first
        cleartext_results = {}

        # Encrypted results
        if self.crypto is not None:
            decryptor = self.crypto["decryptor"]
            encoder = self.crypto["encoder"]

            # Check results
            for var in resolved_variables.items():
                res = decryptor.decrypt(var[1].ciphertext)
                res_dec = list(encoder.decode(res)[0:len(var[1])])
                cleartext_results[var[0]] = res_dec

        # Decrypted results → immediately given
        else:
            for var in resolved_variables.items():
                cleartext_results[var[0]] = resolved_variables[var[0]].vector

        self.result = cleartext_results

    def compute_offloaded_operation(self, operation_input, operation):

        if self.crypto is not None:
            # For later recreating the result with the according EncVariable format
            # First variable always holds a non-constant by defintion
            cleartext_input = []
            crypto_cfg = operation_input.values[0].crypto

            # print("Crypto config", crypto_cfg)

            # Decrypt operation input
            for enc_input in operation_input.values:
                dec_var = self.crypto["decryptor"].decrypt(enc_input.ciphertext)
                dec_plain = self.crypto["encoder"].decode(dec_var)
                res_trunc = dec_plain[0:len(enc_input)]

                # print("GOT OFFL:", res_trunc)

                cleartext_input.append(util.variable.Variable(res_trunc))

            # Create operation input
            cleartext_operation_input = util.atomic.OperationInput(cleartext_input)

            # Perform operation in plain
            result = operation(cleartext_operation_input)

            # print("RESLEN:", len(result))

            # Reencrypt result and pass it back
            plain_result = self.crypto["encoder"].encode(result.vector, self.crypto["scale"])
            enc_result = self.crypto["encryptor"].encrypt(plain_result)
            final_result = util.encrypted_variable.EncVariable(enc_result, crypto_cfg, len(result))

            # Result transmission for bytes counts as well
            # But one offloading is counted as one "transaction"
            if self.config["mode"] == "encrypted":
                evaluation.res["traffic_bytes"] += enc_result.save_size()
                if enc_result.save_size() not in evaluation.res["cipher_size"].keys():
                    evaluation.res["cipher_size"][enc_result.save_size()] = 1
                else:
                    evaluation.res["cipher_size"][enc_result.save_size()] += 1

            return final_result

        # In case of plaintext offloading → compute normally as the proxy would
        else:
            return operation(operation_input)

    def send_inputs(self, proxy):
        crypto = None
        if self.crypto is not None:
            crypto = {
                "relin_keys": self.crypto["relin_keys"],
                "galois_keys": self.crypto["galois_keys"],
                "evaluator": self.crypto["evaluator"],
                "encoder": self.crypto["encoder"],
                "encryptor": self.crypto["encryptor"],
                "scale": self.crypto["scale"],
                "parm_levels": self.crypto["parm_levels"],
                "lowest_parms": self.crypto["lowest_parms"]
            }

        proxy.receive_inputs(self, self.inputs, crypto)

    def prepare_aggregation(self, statistics_server_crypto):

        # Aggregates that get returned to the proxy
        enc_aggregates = {}

        for i in self.result.items():
            result_plain = statistics_server_crypto["encoder"].encode(
                i[1],
                statistics_server_crypto["scale"]
            )
            enc_aggregates[i[0]] = {
                "cipher": statistics_server_crypto["encryptor"].encrypt(result_plain),
                "len": len(i[1])
            }

        return enc_aggregates

    def get_aggregation_result(self, agg_results):
        self.aggregation_results = agg_results
        # print("# Company Evaluation #")
        # for kpi in self.result.items():
        #     print("KPI:", kpi[0], "||OWN:", kpi[1], "|| AVG:", self.aggregation_results[kpi[0]]["avg"], "||")

    def crypto_gen(self, in_polmod=16384, in_level=2, in_scale=pow(2.0, 40)):
        parms = EncryptionParameters(scheme_type.ckks)

        parms.set_poly_modulus_degree(in_polmod)
        levels = [60] + ([40] * (in_level - 1)) + [60]
        parms.set_coeff_modulus(CoeffModulus.Create(in_polmod, levels))

        scale = in_scale
        context = SEALContext(parms)

        # Validate parameters
        if not context.parameters_set():
            seal_util.print_parameters(context, "client")
            raise ValueError("SEAL Context for client not properly configured!")

        keygen = KeyGenerator(context)

        priv_key = keygen.secret_key()
        pub_key = keygen.create_public_key()
        relin_keys = keygen.create_relin_keys()
        galois_keys = keygen.create_galois_keys()

        encryptor = Encryptor(context, pub_key)
        evaluator = Evaluator(context)
        decryptor = Decryptor(context, priv_key)

        encoder = CKKSEncoder(context)
        slot_count = encoder.slot_count()

        parm_levels = seal_util.get_parms(context)
        lowest_parms = seal_util.get_lowest_parms(context)

        # print("Parm levels client", parm_levels)
        # print("Lowest Parms client", lowest_parms)

        # log for eval
        evaluation.res["levels"] = in_level

        return {
            "priv_key": priv_key,
            "pub_key": pub_key,
            "relin_keys": relin_keys,
            "galois_keys": galois_keys,
            "encoder": encoder,
            "scale": scale,
            "slots": slot_count,
            "encryptor": encryptor,
            "evaluator": evaluator,
            "decryptor": decryptor,
            "parm_levels": parm_levels,
            "lowest_parms": lowest_parms
        }
