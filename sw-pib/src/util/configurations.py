import util.constants
import util.fileoperations


class Config:

    def __init__(self, filepath):
        self.config = util.fileoperations.read_yaml(filepath)
        if self.config[util.constants.MODE] in [util.constants.PIB, util.constants.PIBPLUS]:
            self.config[util.constants.ENCRYPTION] = True
        else:
            self.config[util.constants.ENCRYPTION] = False
