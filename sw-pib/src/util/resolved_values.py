# Resolved or "known" computation values
#
# Resolved values are nothing more than a dynamic data structure that holds information on
# the computation of all KPI values of one company. It serves as a lookuptable for existing
# information in which intermediary results are inserted. In the end, the resolved values
# hold all information on the KPIs of one specific company.

class ResolvedValues:

    def __init__(self):
        self.resolved = {}

    # Given a name of the variable check if it is in there
    def has(self, name):
        return name in self.resolved

    # Get a variable from the resolved / computed ones
    def get(self, name):
        if self.resolved[name] is None:
            raise Exception(
                "Could not get the requested variable as it was not computed: ", name)
        return self.resolved[name]

    # Insert a computed variable by its name
    def insert(self, name, var):
        if self.has(name):
            raise Exception("We already computed:", name)
        self.resolved[name] = var

    # Return the list of resolved variables
    def get_resolved(self):
        return self.resolved

    # Remove a list of atomics by their name
    def filter_atomics_by_name(self, atomics):
        for atomic in atomics:
            if self.has(atomic):
                del self.resolved[atomic]

    def print_plain(self):
        print("-" * 10, " VALUES ", "-"*10)
        for (name, val) in self.resolved:
            print("ID: ", name, ", Val: ", val)
        print("-"*28)
