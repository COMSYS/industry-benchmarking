import matplotlib.pyplot as plt
import networkx as nx
import pydot
from networkx.drawing.nx_pydot import graphviz_layout
import pygraphviz as pgv

# assumes a tree structure otherwise it does not terminate


def get_depths_kpis_graphical(algorithms):

    depths = {}
    mul_depths = {}

    # init
    for kpi in algorithms.operations.keys():
        depths[kpi] = 0

    # For each kpi → get the depth
    for kpi in algorithms.operations.values():
        depth = algorithms.get_depths_atomic(kpi["name"])
        depths[kpi["name"]] = depth

        m_depth = algorithms.get_depths_atomic_mult(kpi["name"])
        mul_depths[kpi["name"]] = m_depth

    # print(mul_depths)
    # print(max(mul_depths.values()))

    # Create graph for visualization
    G = pgv.AGraph(directed=True)

    # Generate names
    names = {}
    for i in algorithms.operations.values():
        if depths[i["name"]] is not None:
            names[i["name"]] = i["name"] + " (" + i["op"] + ") " + "@ " + str(depths[i["name"]]) + "@ M" + str(mul_depths[i["name"]])
        else:
            names[i["name"]] = i["name"] + " (" + i["op"] + ") " + "@"

    for i in algorithms.operations.values():
        if (i["op"] == "AdditionConst" and i["constant"] == 0) or i["op"] == "DefConst":
            G.add_node(names[i["name"]], color="gray")
        elif i["op"] == "Multiplication" or i["op"] == "MultiplicationConst":
            G.add_node(names[i["name"]], color="red")
        elif i["is_kpi"] == True:
            G.add_node(names[i["name"]], color="green")
        else:
            G.add_node(names[i["name"]], color="blue")

    for node in algorithms.operations.values():
        for child in node["var"]:
            G.add_edge(names[node["name"]], names[child])

    G.layout(prog="dot")
    G.draw("../data/Results/fileout.pdf")
