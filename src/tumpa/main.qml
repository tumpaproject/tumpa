import QtQuick
import QtQuick.Window
import QtQuick.Controls

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

ApplicationWindow {
    title: qsTr("Tumpa")
    width: 900
    height: 551
    visible: true

    id: root

    // This defines if we will allow saving private key
    property bool allowsecret: false
    // For error message
    property bool keyviewsFlag: false

    ListModel {
        id: ksKeys
    }

    ListModel {
        id: cardList
    }

    SplitView {
        anchors.fill: parent

        // Tags of the left side
        Rectangle {
            color: "#54298B"
            height: root.height
            SplitView.minimumWidth: 217

            Image {
                id: tumpaLogo
                source: "images/logo.svg"
                anchors {
                    topMargin: 14
                    top: parent.top
                    leftMargin: 14
                    left: parent.left
                }
            }

            Column {
                id: menuOptions
                anchors {
                    topMargin: 20
                    top: tumpaLogo.bottom
                    leftMargin: 14
                    left: parent.left
                    right: parent.right
                    rightMargin: 10
                }
                spacing: 4

                LeftIconButton {
                    id: leftIconBttn
                    // This is our initial active menu item
                    active: true
                    anchors.bottomMargin: 18
                    text: qsTr("Key Management")
                    onClicked: {
                        clearActive()
                        active = true
                        // For any other normal views, also remember
                        // to mark the keyviewsFlag as false.
                        keyviewsFlag = false
                        // FIXME: Show only if we have keys
                        stack.pop()
                        gotoKeyList()
                    }
                }

                ColSpacer {
                    height: 10
                }

                LeftIconButton {
                    id: leftKeyBttn
                    // This is our Yubikey button
                    imageSource: "../../images/usbkey.svg"
                    text: qsTr("Smart Card")
                    onClicked: {
                        // First check if we have a card
                        var result = check_if_card()
                        if (result !== true) {
                            // then return
                            return
                        }
                        clearActive()
                        active = true
                        // If we are doing key views, pop the previous view
                        if (keyviewsFlag === true) {
                            stack.pop()
                        }
                        keyviewsFlag = true
                        gotoCardView()
                    }
                }

                LeftMenuButton {
                    id: editNameBttn
                    text: qsTr("Edit Name")
                    onClicked: {
                        // First check if we have a card
                        var result = check_if_card()
                        if (result !== true) {
                            // then return
                            return
                        }
                        clearActive()
                        active = true
                        // If we are doing key views, pop the previous view
                        if (keyviewsFlag === true) {
                            stack.pop()
                        }
                        keyviewsFlag = true
                        stack.push(nameView)
                    }
                }

                LeftMenuButton {
                    id: editPublicURLBttn
                    text: qsTr("Edit Public URL")
                    onClicked: {
                        // First check if we have a card
                        var result = check_if_card()
                        if (result !== true) {
                            // then return
                            return
                        }
                        clearActive()
                        active = true
                        // If we are doing key views, pop the previous view
                        if (keyviewsFlag === true) {
                            stack.pop()
                        }
                        keyviewsFlag = true
                        stack.push(publicurlView)
                    }
                }

                LeftMenuButton {
                    id: editUserPinBttn
                    text: qsTr("Change User Pin")
                    onClicked: {
                        // First check if we have a card
                        var result = check_if_card()
                        if (result !== true) {
                            // then return
                            return
                        }
                        clearActive()
                        active = true
                        // If we are doing key views, pop the previous view
                        if (keyviewsFlag === true) {
                            stack.pop()
                        }
                        keyviewsFlag = true
                        stack.push(userpinView)
                    }
                }

                LeftMenuButton {
                    id: editAdminPinBttn
                    text: qsTr("Change Admin Pin")
                    onClicked: {
                        // First check if we have a card
                        var result = check_if_card()
                        if (result !== true) {
                            // then return
                            return
                        }
                        clearActive()
                        active = true
                        // If we are doing key views, pop the previous view
                        if (keyviewsFlag === true) {
                            stack.pop()
                        }
                        keyviewsFlag = true
                        stack.push(adminpinView)
                    }
                }
            }
            StatusInfo {
                anchors {
                    bottom: parent.bottom
                    bottomMargin: 24
                    left: parent.left
                    leftMargin: 14
                }

                statusText: qsTr("Card detected")
            }
        }

        Rectangle {
            id: bigBox
            color: "white"
            height: root.height
            SplitView.minimumWidth: 683

            StackView {
                id: stack
                initialItem: selectInitialView()
                anchors.fill: parent

                pushEnter: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 50
                    }
                }
                pushExit: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 50
                    }
                }
                popEnter: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 50
                    }
                }
                popExit: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 50
                    }
                }
            }
        }
    }

    function clearActive() {
        for (var i in menuOptions.children) {
            if (!(menuOptions.children[i] instanceof ColSpacer)) {
                menuOptions.children[i].active = false
            }
        }
    }

    Connections {
        target: tbackend
        function onUpdated() {
            stack.pop()
            if (tbackend.haveKeys === true) {
                if (stack.depth === 1) {
                    stack.pop()
                }
                // Get the latest keys
                refreshKeyList()
                stack.push(keylistView)
            }
            //stack.push(ykView)
        }
    }

    Connections {
        target: tbackend
        function onRefreshKeys() {
            // Get the latest keys
            refreshKeyList()
        }
    }

    Component {
        id: startView
        StartView {
            // Do not pop anything, just show proper view
            onGenkeyclicked: stack.push(genkeyView)
        }
    }

    Component {
        id: genkeyView

        GenerateKeyView {
            onNext: {
                stack.pop()
                stack.push(waitView)
                tbackend.generateKey(name, emails, passphrase, expirationDate,
                                     encryptionChecked, signingChecked,
                                     authenticationChecked, keyAlgo)
            }

            onBack: {
                // go back to the last view
                stack.pop()
            }
        }
    }

    Component {
        id: waitView
        WaitView {}
    }

    Component {
        id: keylistView
        KeyListView {
            keyListData: ksKeys
        }
    }

    Component {
        id: cardView

        CardDetailsView {
            cardDetailsList: cardList
        }
    }

    Component {
        id: nameView
        NameView {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateName(name, adminpin)
                if (result === false) {
                    errortext = qsTr("Could not set the name in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: uploadView

        UploadView {

            onNext: {
                // Use the fingerprint variable.
                var whichsubkeys = 0
                if (subkeys.encryptionChecked === true) {
                    whichsubkeys += 1
                }
                if (subkeys.signingChecked === true) {
                    whichsubkeys += 2
                }
                if (subkeys.authenticationChecked === true) {
                    whichsubkeys += 4
                }
                console.log(whichsubkeys)
                // Let us try to format the card and upload
                var result = tbackend.uploadKey(fingerprint, password, true,
                                                whichsubkeys)
                if (result !== "success") {
                    var win = showErrorBox(qsTr("Error in upload"), result)
                    return
                } else {

                    var win2 = getSuccessBox(
                                qsTr("Upload sccessful."), qsTr(
                                    "We successfully uploaded the key to the card."))
                    return
                }
            }
        }
    }

    Component {
        id: publicurlView
        PublicURL {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateURL(url, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set the Public URL in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: userpinView
        UserPin {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateUserPin(userpin, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set the User Pin in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: adminpinView
        AdminPin {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateAdminPin(newpin, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set new Admin Pin in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    function showErrorBox(heading, errortext) {
        // Here we will show the modal dialog to show any error to the user
        var component1 = Qt.createComponent("includes/Utils/InfoModal.qml")
        var win = component1.createObject(root, {
                                              "headingText": heading,
                                              "contentText": errortext
                                          })
        win.okayed.connect(() => {
                               //console.log("Modal okayed")
                               win.destroy()
                           })
        win.show()
        return win
    }

    function getWarningBox(heading, msg) {
        var component1 = Qt.createComponent("includes/Utils/WarningModal.qml")
        var win = component1.createObject(root, {
                                              "dangerBtnText": heading,
                                              "headingText": "Warning",
                                              "contentText": msg
                                          })
        return win
    }

    function getSuccessBox(heading, msg) {
        // When we want to show success operation
        var component1 = Qt.createComponent("includes/Utils/SuccessModal.qml")
        var win = component1.createObject(root, {
                                              "headingText": heading,
                                              "contentText": msg
                                          })
        win.okayed.connect(() => {
                               win.destroy()
                           })
        win.show()
        return win
    }

    function selectInitialView() {
        if (tbackend.haveKeys === true) {
            return keylistView
        } else {
            return startView
        }
    }

    function refreshKeyList() {
        var localdata = tbackend.get_keys_json()
        //console.log(localdata)
        var data = JSON.parse(localdata)
        ksKeys.clear()
        for (var i in data) {
            ksKeys.append(data[i])
        }
    }

    function refreshCardList() {
        var localdata = tbackend.get_card_json()
        //console.log(localdata)
        var data = JSON.parse(localdata)
        cardList.clear()
        for (var i in data) {
            cardList.append(data[i])
        }
    }

    function gotoKeyList() {
        if (tbackend.haveKeys === true) {
            if (stack.depth === 1) {
                stack.pop()
            }
            // Get the latest keys
            refreshKeyList()
            stack.push(keylistView)
        }
    }

    function gotoCardView() {
        if (tbackend.haveCard === true) {
            if (stack.depth === 1) {
                stack.pop()
            }
            // Get the latest information from the card.
            refreshCardList()
            stack.push(cardView)
        }
    }

    function check_if_card() {
        // Checks if we have an smartcard connected or not
        if (tbackend.haveCard !== true) {
            // show error then do nothing
            var errortext = qsTr("Can not access any Yubikey!")
            var win = showErrorBox(qsTr("Error"), errortext)
            return false
        }
        return true
    }
}
