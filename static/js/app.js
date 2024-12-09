window.onload = async function() {
    const response = await fetch('/files');
    const files = await response.json();

    const fileList = document.getElementById('file-list');
    files.forEach(file => {
        const li = document.createElement('li');
        li.innerHTML = `<a href="/download/${file}">${file}</a>`;
        fileList.appendChild(li);
    });
};
