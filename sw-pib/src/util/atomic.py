import enum
import functools
import util.variable
from util.variable import *
from util.encrypted_variable import *
from util.resolved_values import *


# Operation input is nothing more than an ordered list of input variables
class OperationInput:

    # Values is a list of Variables (either of lenght 1, 2, or n)
    # depending on whether the input is unary, binary or nary
    def __init__(self, values):
        self.values = values


# Type definition of output variable
OperationOutput = Variable


# Return operation to compute
def get_op(operation_name: str):
    # somehow match does not work even though i am on version 3.10
    if operation_name == "Addition":
        return add
    elif operation_name == "AdditionConst":
        return add_const
    elif operation_name == "AdditionOverN":
        return add_over_n
    elif operation_name == "Absolute":
        return absolute
    elif operation_name == "DefConst":
        return def_const
    elif operation_name == "Division":
        return div
    elif operation_name == "DivisionConstVar":
        return div_const_var
    elif operation_name == "DivisionVarConst":
        return div_var_const
    elif operation_name == "Maxima":
        return maxima
    elif operation_name == "MaximaOverN":
        return max_over_n
    elif operation_name == "Minima":
        return minima
    elif operation_name == "MinimaOverN":
        return min_over_n
    elif operation_name == "Multiplication":
        return mul
    elif operation_name == "MultiplicationConst":
        return mul_const
    elif operation_name == "Squareroot":
        return sqrt
    elif operation_name == "Subtraction":
        return sub
    elif operation_name == "SubtractionConstVar":
        return sub_const_var
    elif operation_name == "SubtractionVarConst":
        return sub_var_const
    elif operation_name == "Power":
        return power
    elif operation_name == "PowerConst":
        return power_const_exp
    elif operation_name == "PowerBaseConst":
        return power_base_const
    else:
        raise Exception("Did not find operation: ", operation_name)


def get_resolved_for_op(atomic, resolved_vals: ResolvedValues, crypto=None):
    # print("Computing ", atomic["op"], atomic["name"],  "with vars", atomic["var"], "and constant", atomic["constant"])
    # NARY Operations
    if atomic["op"] == "Addition" or atomic["op"] == "Subtraction" or atomic["op"] == "Multiplication" or atomic["op"] == "Minima" or atomic["op"] == "Maxima":
        if len(atomic["var"]) == 0:
            raise Exception("No variable for nary operation")
        if atomic["constant"] is not None:
            raise Exception("Nary operation has unused constant")

        values = []
        # extract the required and resolved values
        for var in atomic["var"]:
            values.append(resolved_vals.get(var))

    # BINARY Operations with 2 Variables
    if atomic["op"] == "Division" or atomic["op"] == "Power":
        if len(atomic["var"]) != 2:
            raise Exception("Binary operation expects exactly two inputs")
        if atomic["constant"] is not None:
            raise Exception("Binary operation has unused constant")

        values = [resolved_vals.get(atomic["var"][0]), resolved_vals.get(atomic["var"][1])]

    # BINARY Operations with one var and one constant on second position
    # BINARY Operations with one var and one constant on the first position
    # In both cases: constant is the second position
    if atomic["op"] == "AdditionConst" or atomic["op"] == "SubtractionVarConst" or atomic["op"] == "MultiplicationConst" or atomic["op"] == "DivisionVarConst" or atomic["op"] == "PowerConst" or atomic["op"] == "PowerBaseConst" or atomic["op"] == "SubtractionConstVar" or atomic["op"] == "DivisionConstVar":
        if atomic["constant"] is None:
            raise Exception("Constant and one var expected but no constant given")
        if len(atomic["var"]) != 1:
            raise Exception("Constant and one var expected but", len(atomic["var"]), "vars provided")

        # For plaintext → create a standard variable
        if crypto is None:
            values = [resolved_vals.get(atomic["var"][0]), Variable([atomic["constant"]])]
        # Otherwise create an encrypted cipher and store it alongside
        else:
            const_cipher = create_constant_cipher(crypto, float(atomic["constant"]))
            const_cipher.plain = float(atomic["constant"])
            values = [resolved_vals.get(atomic["var"][0]), const_cipher]

    # UNARY Operation with no constant
    if atomic["op"] == "Squareroot" or atomic["op"] == "Absolute" or atomic["op"] == "AdditionOverN" or atomic["op"] == "MinimaOverN" or atomic["op"] == "MaximaOverN":
        if atomic["constant"] is not None:
            raise Exception("Constant was not expected for unary operation")
        if len(atomic["var"]) != 1:
            raise Exception("Only one var expected but", len(atomic["var"]), "vars provided")

        values = [resolved_vals.get(atomic["var"][0])]

    # Definition of constants → This makes computation much easier
    if atomic["op"] == "DefConst":
        if len(atomic["var"]) != 0:
            raise Exception("No vars expected for DefConst but", len(atomic["var"]), "vars provided")
        if atomic["constant"] is None:
            raise Exception("Constant was expected for DefConst operation")

        # For plaintext → create a standard variable
        if crypto is None:
            values = [Variable([atomic["constant"]])]

        # Otherwise create an encrypted cipher and store it alongside
        else:
            const_cipher = create_constant_cipher(crypto, float(atomic["constant"]))
            const_cipher.plain = float(atomic["constant"])
            values = [const_cipher]

    # print("Got all values on proxy:", values)
    return values


# Constant definition is nothing more than passing the value through
def def_const(input_var: OperationInput) -> OperationOutput:
    # Pass the variable on
    return input_var.values[0]


# Addition of n variables
def add(input_vars: OperationInput) -> OperationOutput:

    # Create zero value (which is turned into ciphertext / variable)
    zero = None

    if isinstance(input_vars.values[0], EncVariable):
        zero = create_constant_cipher(input_vars.values[0].crypto, 0)
    elif isinstance(input_vars.values[0], Variable):
        zero = Variable([0] * len(input_vars.values[0]))
    else:
        raise Exception("Unknown variable type:", type(input_vars.values[0]))

    # Compute sum over all encrypted variables
    result = functools.reduce(operator.add, input_vars.values, zero)

    return result


# Add constant to var
def add_const(input_vars: OperationInput) -> OperationOutput:

    # Both variable types have a length that is checked beforehand
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")

    # Operator overloading allows for dynamic choosing of implementation
    result = input_vars.values[0] + input_vars.values[1]

    # Control Messages based on variable type
    if isinstance(input_vars.values[0], EncVariable):
        # print("add_const → encrypted output")
        pass
    elif isinstance(input_vars.values[0], Variable):
        # print("got plain vars:", input_vars.values, "sum is", result)
        pass
    else:
        raise Exception("Unknown variable type:", type(input_vars.values[0]))

    return result


# Scalarize vector
def add_over_n(input_var: OperationInput) -> OperationOutput:

    # For both types this is possible without problem
    result = input_var.values[0].add_over_n()

    if isinstance(input_var.values[0], Variable):
        # print("got vars:", input_var.values, "add_over_n is", result)
        pass
    elif isinstance(input_var.values[0], EncVariable):
        # print("got vars:", input_var.values, "add_over_n is encrypted")
        pass

    return result


# Compute min_over_n
def min_over_n(input_vars: OperationInput) -> OperationOutput:
    # print(type(input_vars.values[0]))
    # print("got vars:", input_vars.values, "min_over_n is", input_vars.values[0].min_over_n())
    return input_vars.values[0].min_over_n()


# Compute max_over_n
def max_over_n(input_vars: OperationInput) -> OperationOutput:
    # print("got vars:", input_vars.values, "max_over_n is", input_vars.values[0].max_over_n())
    return input_vars.values[0].max_over_n()


# Add constant to var
def sub(input_vars: OperationInput) -> OperationOutput:

    # print("SUBTRACTING:", input_vars.values)

    # One argument for subtraction → Negation
    if len(input_vars.values) == 1:

        if isinstance(input_vars.values[0], EncVariable):
            return -input_vars.values[0]
        elif isinstance(input_vars.values[0], Variable):
            vec_len = len(input_vars.values[0])
            negation_vec = Variable([-1] * vec_len)
            return input_vars.values[0] * negation_vec

    minuend = input_vars.values[0]
    subtrahends = input_vars.values[1::1]

    result = functools.reduce(operator.sub, subtrahends, minuend)

    if isinstance(input_vars.values[0], Variable):
        # print("got vars:", input_vars.values, "sub is", result)
        pass
    elif isinstance(input_vars.values[0], EncVariable):
        # print("got vars:", input_vars.values, "sub is encrypted")
        pass

    return result


# Add constant to var (the second variable was already turned into a constant)
# The constant is *always* on second place
def sub_const_var(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")
    # print("got vars:", input_vars.values, "sub_const_var is", input_vars.values[1] - input_vars.values[0])
    return input_vars.values[1] - input_vars.values[0]


# The constant is *always* on second place
def sub_var_const(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")
    # print("got vars:", input_vars.values, "sub_var_const is", input_vars.values[0] - input_vars.values[1])
    return input_vars.values[0] - input_vars.values[1]


# Multiply two values
def mul(input_vars: OperationInput) -> OperationOutput:

    # Compute sum over all encrypted variables
    if isinstance(input_vars.values[0], EncVariable):
        one = create_constant_cipher(input_vars.values[0].crypto, 1)
    elif isinstance(input_vars.values[0], Variable):
        one = Variable([1] * len(input_vars.values[0]))
    else:
        raise Exception("Unknown variable type:", type(input_vars.values[0]))

    result = input_vars.values[0] * input_vars.values[1]

    return result


# Add constant to var
def mul_const(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")
    # print("got vars:", input_vars.values, "mul_const is", input_vars.values[0] * input_vars.values[1])
    return input_vars.values[0] * input_vars.values[1]


# Divide one var through another
def div(input_vars: OperationInput) -> OperationOutput:
    # Otherwise use folding and subtract the whole vector
    # print("got vars:", input_vars.values, "div is", input_vars.values[0] / input_vars.values[1])
    return input_vars.values[0] / input_vars.values[1]


# Divide constant through var (the second variable was already turned into a constant)
# The constant is *always* on second place
def div_const_var(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")
    # print("got vars:", input_vars.values, "div_const_var is", input_vars.values[1] / input_vars.values[0])
    return input_vars.values[1] / input_vars.values[0]


# The constant is *always* on second place
def div_var_const(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")

    result = None

    if isinstance(input_vars.values[0], EncVariable):
        # Provide true constant and not the ciphertext
        result = input_vars.values[0].div_var_const(input_vars.values[1].plain)
        # print("got vars:", input_vars.values, "div_var_const is encrypted")
    elif isinstance(input_vars.values[0], Variable):
        result = input_vars.values[0] / input_vars.values[1]
        # print("got vars:", input_vars.values, "div_var_const is ", result)
    else:
        raise Exception("Unexpected variable type: ", type(input_vars.values[0]))

    return result


# Take the square root of a variable
def sqrt(input_vars: OperationInput) -> OperationOutput:
    # print("got vars:", input_vars.values, "sqrt is", input_vars.values[0].sqrt())
    return input_vars.values[0].sqrt()


# Power
def power(input_vars: OperationInput) -> OperationOutput:
    result = input_vars.values[0] ** input_vars.values[1]

    if isinstance(input_vars.values[0], EncVariable):
        # print("got vars:", input_vars.values, "div_var_const is encrypted")
        pass
    elif isinstance(input_vars.values[0], Variable):
        # print("got vars:", input_vars.values, "div_var_const is ", result)
        pass
    else:
        raise Exception("Unexpected variable type: ", type(input_vars.values[0]))

    return result


# Power with constant exponent

def power_const_exp(input_vars: OperationInput) -> OperationOutput:

    result = None

    if isinstance(input_vars.values[0], EncVariable):
        result = input_vars.values[0].power_const_exp(input_vars.values[1].plain)
        # print("got vars:", input_vars.values, "power_const_exp is encrypted")
    elif isinstance(input_vars.values[0], Variable):
        result = input_vars.values[0] ** input_vars.values[1]
        # print("got vars:", input_vars.values, "power_const_exp is ", result)
    else:
        raise Exception("Unexpected variable type: ", type(input_vars.values[0]))

    return result


# Power with constant base
# Constant is *always* the second entry!
def power_base_const(input_vars: OperationInput) -> OperationOutput:
    if len(input_vars.values[0]) != 1:
        raise Exception("Trying to do constant operation with non-scalar!")

    result = input_vars.values[1] ** input_vars.values[0]

    if isinstance(input_vars.values[0], EncVariable):
        # print("got vars:", input_vars.values, "power_base_const is encrypted")
        pass
    elif isinstance(input_vars.values[0], Variable):
        # print("got vars:", input_vars.values, "power_base_const is ", result)
        pass
    else:
        raise Exception("Unexpected variable type: ", type(input_vars.values[0]))

    return result


# Power with constant base
def minima(input_vars: OperationInput) -> OperationOutput:
    # print("got vars:", input_vars.values, "minima is", min(input_vars.values))
    return min(input_vars.values)


# Max of n values
def maxima(input_vars: OperationInput) -> OperationOutput:
    # print("got vars:", input_vars.values, "maxima is", max(input_vars.values))
    return max(input_vars.values)


# Compute absolute
def absolute(input_vars: OperationInput) -> OperationOutput:
    # print("got vars:", input_vars.values, "abs is", abs(input_vars.values[0]))
    return abs(input_vars.values[0])


# Apply binary operation in mapping manner
def map_binary_op(first, second, op):
    if len(first) != len(second):
        raise Exception("Vector dimension missmatch: ", len(first), len(second))

    res = list(map(op, zip(first, second)))
    return util.variable.Variable(res)


# Apply unary operation in mapping manner
def map_unary_op(invar, op):
    res = list(map(op, invar))
    return util.variable.Variable(res)
