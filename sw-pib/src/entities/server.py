from seal import *
from util import seal_util
import os


class StatisticsServer:

    def __init__(self, config):
        self.config = config.config
        self.crypto = self.setup_crypto(self.config["crypto"]["polymod"], self.config["crypto"]["level"], self.config["crypto"]["scale"])

    def setup_crypto(self, in_polmod=16384, in_level=7, in_scale=pow(2.0, 20)):

        parms = EncryptionParameters(scheme_type.ckks)

        parms.set_poly_modulus_degree(in_polmod)
        levels = [60] + ([40] * (in_level - 1)) + [60]  # used to take levels
        parms.set_coeff_modulus(CoeffModulus.Create(in_polmod, levels))

        context = SEALContext(parms)

        # Validate parameters
        if not context.parameters_set():
            seal_util.print_parameters(context, "Statistics Server")
            raise ValueError("SEAL Context not properly configured!")

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

        # print("Parm levels", parm_levels)
        # print("Lowest Parms", lowest_parms)

        return {
            "priv_key": priv_key,
            "pub_key": pub_key,
            "relin_keys": relin_keys,
            "galois_keys": galois_keys,
            "encoder": encoder,
            "scale": in_scale,
            "slots": slot_count,
            "encryptor": encryptor,
            "evaluator": evaluator,
            "decryptor": decryptor,
            "parm_levels": parm_levels,
            "lowest_parms": lowest_parms
        }

    def provide_crypto(self):
        return {
            "pub_key": self.crypto["pub_key"],
            "scale": self.crypto["scale"],
            "relin_keys": self.crypto["relin_keys"],
            "encoder": self.crypto["encoder"],
            "evaluator": self.crypto["evaluator"],
            "scale": self.crypto["scale"],
            "encryptor": self.crypto["encryptor"],
            "parm_levels": self.crypto["parm_levels"],
            "lowest_parms": self.crypto["lowest_parms"]
        }

    def compute_statistics(self, aggregates, k_anonymity):

        decrypted_aggregates = {}
        statistics = {}

        # For each aggregate → min / max / mean
        for agg in aggregates.items():
            decrypted = self.crypto["decryptor"].decrypt(agg[1]["sum"])
            plain = self.crypto["encoder"].decode(decrypted)[0:agg[1]["len"]]
            decrypted_aggregates[agg[0]] = plain

            if agg[1]["comp"] < k_anonymity:
                raise Exception("Statistics server will not release results when less than k (", k_anonymity, ") participants took part in the benchmarking.")

            else:
                statistics[agg[0]] = {
                    "avg": list(map(lambda x: x / agg[1]["comp"], decrypted_aggregates[agg[0]])),
                    "min": 0,  # Comparison not supported
                    "max": 0  # Comparison not supported
                }

        return statistics
