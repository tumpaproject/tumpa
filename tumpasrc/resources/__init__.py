import os

from pkg_resources import resource_filename, resource_string
from PySide2.QtCore import QDir
from PySide2.QtGui import QFontDatabase, QIcon, QPixmap
from PySide2.QtSvg import QSvgWidget

# Add resource directories to the search path.
QDir.addSearchPath("images", resource_filename(__name__, "images"))
QDir.addSearchPath("css", resource_filename(__name__, "css"))


def path(name: str, resource_dir: str = "images/") -> str:
    """
    Return the filename for the referenced image.

    Qt uses unix path conventions.
    """
    return resource_filename(__name__, resource_dir + name)


def load_font(font_folder_name: str) -> None:
    directory = resource_filename(__name__, "fonts/") + font_folder_name
    for filename in os.listdir(directory):
        if filename.endswith(".ttf"):
            QFontDatabase.addApplicationFont(directory + "/" + filename)


def load_icon( iconpath: str) -> QIcon:
    """
    Return a QIcon from the given icon
    """

    icon = QIcon()
    icon.addFile(path(iconpath), mode=QIcon.Normal, state=QIcon.On)
    return icon


def load_svg(name: str) -> QSvgWidget:
    """
    Return a QSvgWidget representation of a file in the resources.
    """
    return QSvgWidget(path(name))


def load_image(name: str) -> QPixmap:
    """
    Return a QPixmap representation of a file in the resources.
    """
    return QPixmap(path(name))


def load_css(name: str) -> str:
    """
    Return the contents of the referenced CSS file in the resources.
    """
    return resource_string(__name__, "css/" + name).decode("utf-8")
