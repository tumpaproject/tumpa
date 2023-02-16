import QtQuick
import QtQuick.Controls

import "includes/Buttons"
import "includes/Utils"

Rectangle {
    property var keyList: null

    id: root
    color: "white"

    signal next

    Rectangle {
        width: root.width
        height: 50

        color: "#E5E7EB"

        Row {
            spacing: 10
            topPadding: 8
            leftPadding: 14

            PrimaryButton {
                labelString: "Generate New Key"
                iconSrc: "../../images/tick_mark.svg"
                isThin: true
            }
            SecondaryButton {
                labelString: "Import Secret Key"
                iconSrc: "../../images/backIcon.svg"
                isThin: true
            }
        }
    }

    ScrollView {
        topPadding: 50
        anchors.fill: parent
        contentWidth: centerColumn.width
        anchors.leftMargin: 14

        Column {
            id: centerColumn
            width: 645
            spacing: 10

            Text {
                id: pathTxt
                text: qsTr("All keys")
                topPadding: 14
                bottomPadding: 10
                font.pixelSize: 20
                font.weight: 700
            }

            ListView {
                width: root.width
                height: keyList.count * 154
                spacing: 10
                model: keyList
                interactive: false
                delegate: KeyItem {
                    width: 645
                    fingerprintTxt: fingerprint
                    createdOnTxt: createdOn
                    expiresOnTxt: expiresOn
                }
            }
        }
    }
}
