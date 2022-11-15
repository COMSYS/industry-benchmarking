import operator
import functools
import math
from seal import *

#########################
# ENCRYPTED COMPUTATION #
#########################
# Since the vector of the plaintext is already padded to a certain length
# we cannot infer their acutal length. However we can infer them at
# decryption since we padded them with 0.
#
# Furthermore: we can do verifications beforehand with the dimensions to not
# run into problems during computation time.

# Variable is the vector of entries that get computed
# Input are 1, 2 or n Variables
# Output is one variable which is "resolved"


class EncVariable:

    def __init__(self, ciphertext, client_crypto, length):
        self.ciphertext = ciphertext
        self.crypto = client_crypto
        self.len = length

    # Addition of vectors

    def __add__(self, other):

        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, max(len(other), len(self)))

        # Rescale to allow computation
        other = result.rescale_ciphers_mult(other)

        # print("ADD: Scale SELF", result.ciphertext.scale(), " and OTHER ", other.ciphertext.scale())

        # Perform operation
        result_cipher = self.crypto["evaluator"].add(result.ciphertext, other.ciphertext)

        self.crypto["evaluator"].relinearize_inplace(result_cipher, self.crypto["relin_keys"])

        result.ciphertext = result_cipher

        return result

    # Subtraction of vectors

    def __sub__(self, other):

        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, max(len(other), len(self)))

        # Rescale to allow computation
        other = result.rescale_ciphers_mult(other)

        # print("SUB: Scale SELF", result.ciphertext.scale(), " and OTHER ", other.ciphertext.scale())

        # Perform operation
        result_cipher = self.crypto["evaluator"].sub(result.ciphertext, other.ciphertext)

        self.crypto["evaluator"].relinearize_inplace(result_cipher, result.crypto["relin_keys"])

        result.ciphertext = result_cipher

        # print("[Scale, Size] of fixed result is: [", result.ciphertext.scale(), ", ", result.ciphertext.size(), "]")

        return result

    # Negation of variable

    def __neg__(self):

        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, len(self))

        result_cipher = result.crypto["evaluator"].negate(self.ciphertext)
        result.ciphertext = result_cipher

        # Relinearize as it is basically
        result.crypto["evaluator"].relinearize_inplace(result.ciphertext, result.crypto["relin_keys"])
        result.crypto["evaluator"].rescale_to_next_inplace(result.ciphertext)

        # print("[Scale, Size] of fixed result is: [", result.ciphertext.scale(), ", ", result.ciphertext.size(), "]")

        return result

    # Multiplication of vectors

    def __mul__(self, other):

        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, max(len(other), len(self)))
        # print("MUL LEN", max(len(other), len(self)))

        # Rescale to allow computation
        other = result.rescale_ciphers_mult(other)

        # Perform operation
        result_cipher = result.crypto["evaluator"].multiply(result.ciphertext, other.ciphertext)

        self.crypto["evaluator"].relinearize_inplace(result_cipher, result.crypto["relin_keys"])
        self.crypto["evaluator"].rescale_to_next_inplace(result_cipher)

        result.ciphertext = result_cipher

        # print("[Scale, Size] of fixed result is: [", result.ciphertext.scale(), ", ", result.ciphertext.size(), "]")

        return result

    # Truedivision of vectors

    def __truediv__(self, other):
        self.offload_operation()

    # Power of vectors

    def __pow__(self, other):
        self.offload_operation()

    # Compute Absolute of vector

    def __abs__(self):
        self.offload_operation()

    # Squareroot of vector

    def sqrt(self):
        self.offload_operation()

    # Find the minimum value, which returns a scalar (still in vector)

    def min_over_n(self):
        self.offload_operation()

    # Find the maximum value, which returns a scalar (still in vector)

    def max_over_n(self):
        self.offload_operation()

    # Division with constant is possible

    def div_var_const(self, constant):
        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, len(self))

        # create constant ciphertext for divisor
        constant_cipher = create_constant_cipher(self.crypto, float(1/constant))

        # Rescale to allow computation
        other = result.rescale_ciphers_mult(constant_cipher)

        # Perform operation
        result_cipher = result.crypto["evaluator"].multiply(result.ciphertext, constant_cipher.ciphertext)
        result.ciphertext = result_cipher

        # print("[Scale, Size] of fixed result is: [", result.ciphertext.scale(), ", ", result.ciphertext.size(), "]")

        return result

    # Exponentiation with SEAL only allows to
    # use integers for exponentiation and
    # especially not ciphertexts

    def power_const_exp(self, constant):
        # Do not modify the variable itself as it
        # may be used later on again
        result = EncVariable(self.ciphertext, self.crypto, len(self))

        # Use self as it is the initial result
        # -1 is used since we have implicitly * 1
        # additionally this is better for scale issues
        for i in range(int(constant) - 1):
            result *= self

        return result

    # Scalarize vector to get result (still in vector)
    # Works with galois keys by rotating for the amount
    # of length given
    # → Mean computation is possible without a statistics
    # server by multiplying with constant 1/k

    def add_over_n(self):

        result = EncVariable(self.ciphertext, self.crypto, len(self))

        result_cipher = result.ciphertext

        slots = len(self)
        iterations = int(math.log(slots, 2))
        if slots != 2 ** iterations:
            iterations = iterations + 1

        # We do only log_2(slots steps)
        # because we add up 2 ** i slots in each
        # step → we finish after log_2(n) steps
        # 1 2 3 4
        # First round
        # → 1 2 3 4
        # → 2 3 4 1
        # ----------
        # → 3 5 7 5
        # This is the sum of the field itself and the preceding one
        #
        # Second round: Now adding up the two fields preceding fields up
        # → 3 5 7 5
        # → 7 5 3 5
        # ----------
        # → 10 10 10 10
        # All have the sum over the entire vector
        for i in range(iterations):
            rotation_vec = result.crypto["evaluator"].rotate_vector(
                result_cipher,
                2 ** i,
                result.crypto["galois_keys"]
            )
            result.crypto["evaluator"].add_inplace(
                result_cipher,
                rotation_vec
            )
        # Scalarizing means that our vector is reduced to length 1
        result = EncVariable(result_cipher, self.crypto, 1)
        return result

    ##############################
    # Comparators don't exist    #
    # since FHE does not support #
    # this operation type        #
    ##############################

    # It seems to be possible
    # However implementing this allows
    # to guess the numbers by comparing
    # them to another, as far as i
    # understood from skimming this:
    # https://eprint.iacr.org/2019/417.pdf

    #########################################
    # Other helpful "dunders" and functions #
    #########################################

    # Get length of variable which is the dimension
    def __len__(self):
        return self.len

    # Printing the string representation does not help
    def __repr__(self):
        outstr = "EncVariable [len: " + str(len(self)) + "]"
        # outstr = "EncVariable: [" + str(self.ciphertext.to_string()) + "]"
        return outstr

    # Perform rescaling of ciphers (until not possible anymore)
    def rescale_ciphers_mult(self, other_cipher):

        # print("##### RESCALE #####")
        (parms, mags) = self.cipher_statistics(other_cipher)

        # print("magnitudes", mags)
        # print("parms", parms)

        # First we rescale to make sure our scales don't exceed the bounds
        max_scale = len(self.crypto["parm_levels"]) * 40  # 40 represents the configured value
        # print(mags, parms, max_scale)

        while sum(mags) > max_scale:
            # Look for the value with the highest level/lowest prms
            index = 0 if parms[0] < parms[1] else 1
            # print("INDEX", index)
            if index == 0:
                self.crypto["evaluator"].rescale_to_next_inplace(self.ciphertext)
            elif index == 1:
                self.crypto["evaluator"].rescale_to_next_inplace(other_cipher.ciphertext)
            else:
                print("FATAL: could not rescale index", index)

            # Update the statistics
            (parms, mags) = self.cipher_statistics(other_cipher)

        # Make sure scales match
        index = 0 if parms[0] < parms[1] else 1
        # print("LOWEST MAGNITUDE INDEX (0 = SELF) (1 = OTHER):", index, "(REF: ", parms, ")")
        lowest_magnitude = None

        if index == 0:
            lowest_magnitude = magnitude(self.ciphertext.scale())
        elif index == 1:
            lowest_magnitude = magnitude(other_cipher.ciphertext.scale())

        # print("Res:", lowest_magnitude, "mags", mags)

        while not mags[1-index] == lowest_magnitude:
            if index == 0:
                self.crypto["evaluator"].rescale_to_next_inplace(self.ciphertext)
            elif index == 1:
                self.crypto["evaluator"].rescale_to_next_inplace(other_cipher.ciphertext)
            else:
                print("FATAL: cannot rescale index", index)

            # Update the statistics
            (parms, mags) = self.cipher_statistics(other_cipher)

        # Determine lowest parms level
        index = 0
        # print('Parm levels', self.crypto["parm_levels"])
        # print('Current parms', parms)

        lowest = 0
        for i in range(len(parms)):
            if self.crypto["parm_levels"][parms[i]] > lowest:
                index = i
                lowest = self.crypto["parm_levels"][parms[i]]

        # print("LOWEST INDEX", index)

        if index == 0:
            lowest_parms = self.ciphertext.parms_id()
        elif index == 1:
            lowest_parms = other_cipher.ciphertext.parms_id()
        else:
            print("FATAL: could not find index for lowest parm", index)

        # print("SELF at LEVEL [", self.crypto["parm_levels"][parms[0]],
        #       "]", "OTHER at LEVEL [", self.crypto["parm_levels"][parms[1]],
        #       "]", "with lowest of them being",
        #       "SELF" if index == 0
        #       else {"NONE" if self.crypto["parm_levels"][parms[0]] == self.crypto["parm_levels"][parms[1]]
        #             else "OTHER"})
        # print('Lowest parms', lowest_parms[0])
        # print("Switching to this lowest parm for the other cipher!")

        self.crypto["evaluator"].mod_switch_to_inplace(self.ciphertext, lowest_parms)
        self.crypto["evaluator"].mod_switch_to_inplace(other_cipher.ciphertext, lowest_parms)

        # print("Both were switched to LEVEL [", self.crypto["parm_levels"][lowest_parms[0]], "]")

        # print("Switched  modulus!")
        # print("now parms and mags")
        # print(self.cipher_statistics(other_cipher))

        # print("Relinearizing..")

        self.crypto["evaluator"].relinearize_inplace(self.ciphertext, self.crypto["relin_keys"])
        self.crypto["evaluator"].relinearize_inplace(other_cipher.ciphertext, self.crypto["relin_keys"])

        self.ciphertext.scale(pow(2.0, magnitude(self.ciphertext.scale())))
        other_cipher.ciphertext.scale(pow(2.0, magnitude(other_cipher.ciphertext.scale())))

        # print("Relinearization complete!")
        # print("##### RESCALE DONE #####")

        return other_cipher

    def offload_operation(self):
        raise Exception("Offloading operation")

    # Returns dicts for parms and magnitudes for each argument
    def cipher_statistics(self, other):

        resparms = []
        resmags = []

        resparms.append(self.ciphertext.parms_id()[0])
        resparms.append(other.ciphertext.parms_id()[0])
        resmags.append(magnitude(self.ciphertext.scale()))
        resmags.append(magnitude(other.ciphertext.scale()))

        return (resparms, resmags)


# Create a ciphertext from a constant value
def create_constant_cipher(crypto, constant):
    constant_encoded = crypto["encoder"].encode([constant], crypto["scale"])
    constant_cipher = crypto["encryptor"].encrypt(constant_encoded)
    constant_len = 1
    return EncVariable(constant_cipher, crypto, constant_len)


# Computes order of magnitude in 2-log
def magnitude(x):
    return int(math.log2(x))
