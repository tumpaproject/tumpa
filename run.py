#!/usr/bin/env python3

import os
import sys
import inspect

path = os.path.join(os.path.dirname(os.path.abspath(inspect.getfile(inspect.currentframe()))), "src/")

sys.path.insert(0, path)

import tumpa

tumpa.main()
