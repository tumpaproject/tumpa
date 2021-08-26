import QtQuick 2.0

Rectangle {
    id: saveButton
    color: "skyblue"
    width: 160
    height: 40
    border.color: "gainsboro"
    // In case you want to change the button text
    property alias button_txt: buttonTxt.text

    // This signal will be fired when the user will click the button
    signal clicked


    Text {
        id: buttonTxt
        anchors.centerIn: parent
        text: "Save Changes"
    }
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true

        onEntered: {
            saveButton.border.color = "black"
        }
        onExited: {
            saveButton.border.color = "gainsboro"
        }
        onClicked: {
            saveButton.clicked()
        }
    }
}
