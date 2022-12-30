import QtQuick
import QtQuick.Window
import QtQuick.Controls

import "../Buttons"

Window {
    id: root
    flags: Qt.Dialog
    width: 512
    height: 200
    modality: Qt.WindowModal

    signal accepted
    signal rejected

    property alias dangerBtnText: dangerButton.labelString
    property alias headingText: modalHeading.text
    property alias contentText: modalText.text

    Row {
        anchors {
            topMargin: 36
            leftMargin: 24
            top: parent.top
            left: parent.left
        }
        width: root.width - 48
        spacing: 16

        Image {
            id: icon
            source: "../../images/warning.svg"
            width: 40
            height: 40
            fillMode: Image.PreserveAspectFit
            sourceSize.width: 1024
            sourceSize.height: 1024
            mipmap: true
        }

        Column {
            width: parent.width - 56
            Text {
                id: modalHeading
                color: "#111827"
                font.pixelSize: 18
                font.weight: 500
                lineHeight: 1.3
            }
            Text {
                id: modalText
                color: "#5F6672"
                width: parent.width
                font.pixelSize: 14
                font.weight: 400
                wrapMode: Text.WordWrap
            }
        }
    }

    Row {
        anchors {
            bottom: parent.bottom
            right: parent.right
            bottomMargin: 32
            rightMargin: 24
        }
        spacing: 12

        SecondaryButton {
            labelString: qsTr("Cancel")
            onClicked: root.rejected()
        }
        ModalDangerButton {
            id: dangerButton
            labelString: qsTr("Delete")
            onClicked: root.accepted()
        }
    }
}
