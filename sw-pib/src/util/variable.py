import operator
import functools
import math
from numpy import float64

#####################
# PLAIN COMPUTATION #
#####################
# Variable is the vector of entries that get computed
# Input are 1, 2 or n Variables
# Output is one variable which is "resolved"


class Variable:

    def __init__(self, values):
        self.vector = values

    # Addition of vectors

    def __add__(self, other):
        return Variable(list(map(lambda x: x[0] + x[1], zip(self.vector, other.vector))))

    # Subtraction of vectors

    def __sub__(self, other):
        return Variable(list(map(lambda x: x[0] - x[1], zip(self.vector, other.vector))))

    # Multiplication of vectors

    def __mul__(self, other):
        return Variable(list(map(lambda x: x[0] * x[1], zip(self.vector, other.vector))))

    # Truedivision of vectors

    def __truediv__(self, other):
        return Variable(list(map(lambda x: x[0] / x[1], zip(self.vector, other.vector))))

    # Floordivision of vectors

    def __floordiv__(self, other):
        return Variable(list(map(lambda x: x[0] // x[1], zip(self.vector, other.vector))))

    # Power of vectors

    def __pow__(self, other):
        # Use standard computation on invalid input
        return Variable(list(map(pow_helper, zip(self.vector, other.vector))))

    # Compute Absolute of vector

    def __abs__(self):
        return Variable(list(map(lambda x: abs(x), self.vector)))

    # Squareroot of vector
    # INFO: IKV has an overflow regarding the scale → we use abs
    # to overcome this issue, however the result in the computation
    # is incorrect due to this.
    def sqrt(self):
        return Variable(list(map(lambda x: math.sqrt(abs(x)), self.vector)))

    # Find the minimum value, which returns a scalar (still in vector)

    def min_over_n(self):
        x = min(self.vector)
        return Variable([x])

    # Find the maximum value, which returns a scalar (still in vector)

    def max_over_n(self):
        x = max(self.vector)
        return Variable([x])

    # Scalarize vector to get result (still in vector)

    def add_over_n(self):
        x = functools.reduce(operator.add, self.vector, 0)
        return Variable([x])

    #############################
    # Comparators for min + max #
    #############################

    # Less than of vectors
    def __lt__(self, other):
        if len(self) != len(other):
            raise Exception("No meaningful comparison possible - dimension missmatch: ", self, " || ", other)

        tups = list(zip(self.vector, other.vector))
        for i in tups:
            if i[0] < i[1]:
                return True
            if i[0] > i[1]:
                return False

        # They are the same
        return False

    # Greater than of vectors
    def __gt__(self, other):
        if len(self) != len(other):
            raise Exception("No meaningful comparison possible - dimension missmatch: ", self, " || ", other)

        tups = list(zip(self.vector, other.vector))
        for i in tups:
            if i[0] > i[1]:
                return True
            if i[0] < i[1]:
                return False

        # They are the same
        return False

    # Less than or equal of vectors
    def __le__(self, other):
        return not self.__gt__(other)

    # Greater than or equal of vectors
    def __ge__(self, other):
        return not self.__lt__(other)

    # Equal of vectors
    def __eq__(self, other):
        if len(self) != len(other):
            raise Exception("No meaningful comparison possible - dimension missmatch: ", self, " || ", other)

        tups = list(zip(self.vector, other.vector))
        for i in tups:
            if i[0] != i[1]:
                return False

        # They are the same
        return True

    # Not equal of vectors
    def __ne__(self, other):
        return not self.__eq__(other)

    ########################
    # Other helpful "dunders"
    ########################

    # Get length of variable which is the dimension
    def __len__(self):
        return len(self.vector)

    def __repr__(self):
        outstr = "Variable: ["
        for index in range(len(self)):
            outstr += str(self.vector[index])
            if index != len(self) - 1:
                outstr += ", "

        outstr += "]"
        return outstr


# For circumventing runtime errors with negative
# bases and fractional exponents
def pow_helper(input_tup):
    if input_tup[0] < 0 and type(input_tup[1]) == float64:
        return math.pow(float(input_tup[0]), int(input_tup[1]))
    else:
        return math.pow(float(input_tup[0]), float(input_tup[1]))
