import yaml
import os


def read_yaml(filepath):
    with open(filepath, 'r') as stream:
        return yaml.safe_load(stream)


def get_all_yaml_files_in_dir(path):
    files = []

    for filename in os.listdir(path):
        file = os.path.join(path, filename)
        if os.path.isfile(file) and file.endswith(".yaml"):
            files.append(file)

    return sorted(files)
