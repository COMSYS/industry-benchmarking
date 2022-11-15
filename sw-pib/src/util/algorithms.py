import util.fileoperations
import util.representation


class Algorithm:

    def __init__(self, filepath):
        self.kpis = []
        self.non_kpis = []
        self.required = {}
        self.intervals = None
        self.parse_algorithms(filepath)

    # Read operations from input
    def parse_algorithms(self, filepath):
        self.operations = {}
        atomic_ops = util.fileoperations.read_yaml(filepath)

        # Store the input in an internal "hashmap" data structure
        for entry in atomic_ops['operations']:
            name = entry['name']
            self.operations[name] = {
                "name": entry["name"],
                "op": entry["op"],
                "is_kpi": entry["is_kpi"],
                "var": entry["var"],
                "constant": entry.get("constant", None)
            }

        # After setting it up → get required inputs
        self.get_required_inputs()

        # Measure depth and show graphically
        # self.get_depths_kpis()
        util.representation.get_depths_kpis_graphical(self)

        # Perform topological sorting on the operations
        self.topological_op_sort()

    def get_required_inputs(self):
        # Initialize the required vars as empty
        self.required = {}

        # Sort the ops into categories
        for i in self.operations.values():
            if i["is_kpi"]:
                self.kpis.append(i)
            else:
                self.non_kpis.append(i["name"])

        # extend dependency graph for vars that are not ops
        for op in self.operations.copy().values():
            for subop in op["var"]:
                if not self.has_atomic_var(subop):
                    # We identify a required variable
                    required_input_atom = new_required_atomic(subop)

                    # Insert it into the lists and extensions
                    self.required[subop] = required_input_atom
                    self.non_kpis.append(subop)
                    self.operations[subop] = required_input_atom

    # Given a name, this returns whether an atomic with this name exists and returns a copy of it
    def find_atomic_by_name(self, id):
        return self.operations[id]

    # Given an name return whether an atomic with this name exists
    def has_atomic_var(self, id):
        if id in self.operations:
            return True
        return False

    # Return the sorted list of operations that have to be computed procedually
    # In case of dependency errors it reports them
    def topological_op_sort(self):
        # Map for ordering
        topo = {}

        # Initialize it with unresolved and depth 0
        for i in self.operations.values():
            topo[i["name"]] = [0, "Unresolved"]

        topo_num = 0
        topo = self.dfs_topo_sort(topo, topo_num)

        # print("Topological Order:", topo)

        resolution_ordering = []
        # Remove input vars as they already exist
        currated_vars = list(topo.copy().items())
        for i in currated_vars.copy():
            if self.required.get(i[0]) == None:
                resolution_ordering.append(i)

        # Sort results and return the operations in topological ordering
        resolution_ordering = sorted(
            resolution_ordering, key=lambda x: x[1][0], reverse=False)
        # print("Currated Order", resolution_ordering)

        resolution_ordering = list(map(lambda x: x[0], resolution_ordering))
        op_cpy = self.operations.copy()
        self.operations = list(map(lambda x: [x, op_cpy[x]], resolution_ordering))

    # Caller function for all KPIs in case of a dependency graph consisting of multiple trees

    def dfs_topo_sort(self, topo, topo_num):
        for i in self.operations.copy().values():
            if topo.get(i["name"])[1] == "Unresolved":
                (topo, topo_num) = self.dfs_topo_sort_inner(
                    topo, i["name"], topo_num)
                # print(topo)
        return topo

    # Handle topological sorting on one tree
    def dfs_topo_sort_inner(self, topo, curr_op, topo_num):
        tup = topo[curr_op]
        tup[1] = "InVisit"

        # Iterate through children
        for i in self.find_atomic_by_name(curr_op)["var"]:
            if topo.get(i)[1] == "InVisit":
                raise Exception("Cyclic dependency in algorithms")
            elif topo.get(i)[1] == "Unresolved":
                # Recursively go deeper
                (topo, topo_num) = self.dfs_topo_sort_inner(topo, i, topo_num)
            elif topo.get(i)[1] == "Resolved":
                # OK nothing to do
                pass
            else:
                raise Exception("State is undefined")

        topo[curr_op][0] = topo_num
        topo[curr_op][1] = "Resolved"
        topo_num += 1
        return [topo, topo_num]

    # assumes a tree structure otherwise it does not terminate
    def get_depths_kpis(self):

        kpi_depths = {}

        # For each kpi → get the depth
        for kpi in filter(lambda x: x["is_kpi"], self.operations.values()):
            depth = 0
            for atomic in kpi["var"]:
                depth = max(depth, self.get_depths_atomic(atomic, 0))

            # Count one up to attribute the inputs which require to be comupted
            depth += 1

            # Store final depth
            kpi_depths[kpi["name"]] = depth

        # print(kpi_depths)
        # print("avg:", str(sum(kpi_depths.values()) / len(kpi_depths.values())), "max:", str(max(kpi_depths.values())))

    def get_depths_atomic(self, node):

        if len(self.find_atomic_by_name(node)["var"]) == 0:
            return 0
        else:

            # Initialize empty maximum
            depth_list = []

            for child in self.find_atomic_by_name(node)["var"]:
                depth_list.append(self.get_depths_atomic(child))

            return (max(depth_list) + 1)

    def get_depths_atomic_mult(self, node):

        if len(self.find_atomic_by_name(node)["var"]) == 0:
            return 0
        else:

            # Initialize empty maximum
            depth_list = []

            for child in self.find_atomic_by_name(node)["var"]:

                childdepth = self.get_depths_atomic_mult(child)
                if self.find_atomic_by_name(child)["op"] in ["MinimaOverN", "MaximaOverN", "DivisionConstVar", "Squareroot", "Power", "PowerBaseConst",
                                                             "Division", "Minima", "Maxima", "Division", "Absolute"]:
                    childdepth = 0
                depth_list.append(childdepth)

            if self.find_atomic_by_name(node)["op"] == "Multiplication" or self.find_atomic_by_name(node)["op"] == "MultiplicationConst":
                return (max(depth_list) + 1)
            else:
                return max(depth_list)


def new_required_atomic(name):
    return {
        "name": name,
        "is_kpi": False,
        "op": "AdditionConst",
        "var": [],
        "constant": 0,
    }
