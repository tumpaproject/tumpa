import os

import setuptools

with open("README.md", "r") as fh:
    long_description = fh.read()

# The CSS file
package_resources = ["src/tumpa/resources/css/mainwindow.css"]

# All other graphics used in the client
for name in os.listdir("./src//tumpa/resources/images/"):
    package_resources.append(os.path.join("./src/tumpa/resources/images", name))

setuptools.setup(
    name="tumpa",
    version="0.1.3",
    author="Kushal Das",
    author_email="mail@kushaldas.in",
    description="OpenPGP key creation and smartcard access.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    license="GPLv3+",
    python_requires=">=3.5",
    url="https://github.com/kushaldas/tumpa",
    packages=["src/tumpa", "src/tumpa.resources"],
    include_package_data=True,
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Programming Language :: Python :: 3",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "License :: OSI Approved :: GNU General Public License v3 or later (GPLv3+)",
        "Intended Audience :: Developers",
        "Operating System :: OS Independent",
        ],
    entry_points={"console_scripts": ["tumpa = tumpa:main"]},
)
