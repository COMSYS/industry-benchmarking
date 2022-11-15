from seal import scheme_type


'''
Print SEAL context configuration
'''


def get_parms(context):
    context_data = context.first_context_data()
    parms = {}
    i = 1
    while(context_data):
        parms[context_data.parms_id()[0]] = i
        i += 1
        context_data = context_data.next_context_data()
    return parms


def get_lowest_parms(context):
    context_data = context.first_context_data()
    lowest_parms_id = None
    while(context_data):
        lowest_parms_id = context_data.parms_id()
        context_data = context_data.next_context_data()
    return lowest_parms_id


def print_parameters(context, name):
    context_data = context.key_context_data()
    if context_data.parms().scheme() == scheme_type.bfv:
        scheme_name = 'bfv'
    elif context_data.parms().scheme() == scheme_type.ckks:
        scheme_name = 'ckks'
    else:
        scheme_name = 'none'
    print('== SEAL PARAMS [', name, ']==')
    print('Configuration OK: ', context.parameters_set())
    print('Encryption scheme: ' + scheme_name)
    print(f'poly_modulus_degree: {context_data.parms().poly_modulus_degree()}')
    coeff_modulus = context_data.parms().coeff_modulus()
    coeff_modulus_sum = 0
    for j in coeff_modulus:
        coeff_modulus_sum += j.bit_count()
    print(f'coeff_modulus size: {coeff_modulus_sum} (', end='')
    for i in range(len(coeff_modulus) - 1):
        print(f'{coeff_modulus[i].bit_count()} + ', end='')
    print(f'{coeff_modulus[-1].bit_count()}) bits')
    print('===========================')


def print_vector(vec, print_size=4, prec=3):
    slot_count = len(vec)
    print()
    if slot_count <= 2*print_size:
        print('    [', end='')
        for i in range(slot_count):
            print(f' {vec[i]:.{prec}f}' +
                  (',' if (i != slot_count - 1) else ' ]\n'), end='')
    else:
        print('    [', end='')
        for i in range(print_size):
            print(f' {vec[i]:.{prec}f},', end='')
        if slot_count > 2*print_size:
            print(' ...,', end='')
        for i in range(slot_count - print_size, slot_count):
            print(f' {vec[i]:.{prec}f}' +
                  (',' if (i != slot_count - 1) else ' ]\n'), end='')
    print()


def modulus_chain(self):
    print("..............................................")
    print("Modulus switching chain")
    print("..............................................")

    # First print the key level parameter information.

    context_data = self.context.key_context_data()
    print("----> Level (chain index): ", context_data.chain_index())
    print(" ...... key_context_data()")
    print("      parms_id: ", context_data.parms_id())
    print("      coeff_modulus primes: ")
    print(hex)
    for prime in context_data.parms().coeff_modulus():
        print(prime.value(), " ")

    # print(dec)
    print("\\")
    print(" \\-->")

    context_data = self.context.first_context_data()
    while (context_data):
        print(" Level (chain index): ", context_data.chain_index())
        if (context_data.parms_id() == self.context.first_parms_id()):
            print(" ...... first_context_data()")

        elif (context_data.parms_id() == self.context.last_parms_id()):
            print(" ...... last_context_data()")

        print("      parms_id: ", context_data.parms_id())
        print("      coeff_modulus primes: ")
        print(hex)
        for prime in context_data.parms().coeff_modulus():
            print(prime.value(), " ")

    #     print(dec)
        print("\\")
        print(" \\-->")

        # Step forward in the chain.

        context_data = context_data.next_context_data()
    print("..............................................")
    print(" End of chain reached")
    print("..............................................")
