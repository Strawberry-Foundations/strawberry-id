function unfocus(element) {
    document.getElementById(element).addEventListener('blur', function() {
        this.classList.add('input-field--unfocus');
    });
}

function hideElement(element) {
    var label = document.getElementById(element);
    label.classList.add("hidden");
}

function showElement(element) {
    var label = document.getElementById(element);
    label.classList.remove("hidden");
}

/* 
const inputField = document.getElementById('inputField');
inputField.addEventListener('focus', () => {
    inputField.classList.add('active');
});
inputField.addEventListener('blur', () => {
    if (!inputField.value.trim()) {
        inputField.classList.remove('active');
    }
});
*/