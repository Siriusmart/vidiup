const instance = document.getElementById("instance");
const preview = document.getElementById("preview");
const add = document.getElementById("add");
const mainscreen = document.getElementById("mainscreen");
const region = document.getElementById("region");

function getAddress() {
    return instance.value.split("://").pop().split("/").shift().trim();
}

function isValid() {
    let address = getAddress().split(".");
    return (
        address.length > 1 &&
        !address.some((val) => val.length === 0 || val.includes(" "))
    );
}

function updatePreview() {
    let newPreview = document.createElement("i");
    let adderss = getAddress();

    if (adderss.length === 0) {
        newPreview.innerText = "[empty]";
    } else {
        newPreview.innerText = adderss;
    }

    if (isValid()) {
        add.classList.remove("invalid");
    } else {
        newPreview.classList.add("invalid");
        add.classList.add("invalid");
    }

    preview.innerHTML = `Instance: ${newPreview.outerHTML}`;
}

updatePreview();

instance.addEventListener("input", () => {
    updatePreview();
});

function request() {
    fetch(`/api/v1/add?region=${region.value}&instance=${getAddress()}`)
        .then((response) => {
            return response.json();
        })
        .catch((e) => {
            console.log(e);
            alert(
                "API Error, cannot decode JSON data. Check console for response detail.",
            );
        });
}

add.addEventListener("click", () => {
    if (add.classList.contains("invalid")) {
        return;
    }

    mainscreen.classList.add("click");

    let I = 300;
    let k = instance.value.length;
    let m = -Math.log2(k) / I;

    let nextLength = k - 1;
    let begin = new Date().getTime();

    function reduce() {
        let nextTime = Math.log2(nextLength / k) / m;
        nextLength -= 1;

        setTimeout(
            () => {
                instance.value = instance.value.substring(0, nextLength);

                if (nextLength < 1) {
                    setTimeout(() => {
                        instance.value = "";
                        updatePreview();
                    }, 100);
                } else {
                    reduce();
                }
            },
            begin + nextTime - new Date().getTime(),
        );
        updatePreview();
    }

    reduce();

    setTimeout(() => {
        mainscreen.classList.remove("click");
    }, 500);

    request();
});
