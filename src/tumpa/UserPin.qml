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
    property alias userpin: userPinTxt.text

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
            text: qsTr("Change User Pin")
            bottomPadding: 10
            font.pixelSize: 20
            font.weight: 700
        }

        PasswordInputField {
            id: adminPinTxt
            width: 645
            height: 40
            labelString: qsTr("Current Admin Pin")
            pin: true
        }

        Item {
            width: parent.width
            height: 20
        }

        PasswordInputField {
            id: userPinTxt
            width: 645
            height: 40
            labelString: qsTr("New User Pin")
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
            console.log(userPinTxt.text)
        }
    }
}
