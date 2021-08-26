import QtQuick 2.0

Item {

    Rectangle {
        id: root
        width: 800
        height: 600
        color: "white"

        Row {

            USaveButton {
                id: generateKeyButton
                button_txt: "Create New Key"


            }
        }
    }
}
