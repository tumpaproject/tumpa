import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "components/Forms"
import "components/Buttons"
import "components/Utils"

Rectangle {
    id: root
    color: "white"
    property alias adminpin: adminPinTxt.text
    property alias url: urlTxt.text

    signal next

    Column {
        id: centerColumn
        anchors.top: root.top
        anchors.left: root.left
        anchors.right: root.right
        anchors.leftMargin: 14
        spacing: 10

        Item {
            width: parent.width
            height: 24
        }

        Text {
            id: pathTxt
            text: qsTr("Change Public URL")
            bottomPadding: 10
            font.pixelSize: 20
            font.weight: 700
        }

        TextInputField {
            id: urlTxt
            width: 645
            height: 40
            labelString: qsTr("Public URL")
            // This helps to set the current height of the box
            pin: true
        }

        Item {
            width: parent.width
            height: 20
        }

        PasswordInputField {
            id: adminPinTxt
            width: 645
            height: 40
            labelString: qsTr("Current Admin Pin")
            pin: true
        }
    }

    PrimaryButton {
        id: saveKeyBttn
        anchors {
            bottom: root.bottom
            right: root.right
            rightMargin: 24
            bottomMargin: 24
        }
        labelString: qsTr("Save")
        iconSrc: "../../images/tick_mark.svg"

        onClicked: {
            console.log(adminPinTxt.text)
            console.log(urlTxt.text)
        }
    }
}
