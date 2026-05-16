function initializeTitlebar(options = { minimizeOnly: false }) {
    const titlebar = document.querySelector('.electros-titlebar');
    const platform = navigator.userAgent.includes('Mac') ? 'mac' : 
                     navigator.userAgent.includes('Win') ? 'win' :
                     'linux';

    titlebar.classList.add(platform);
    titlebar.setAttribute('data-tauri-drag-region', 'deep');

    const buttonsAlignClass = platform === 'mac' ? 'electros-titlebar-buttons-align-left' : 'electros-titlebar-buttons-align-right';

    if (platform === 'mac') {
        titlebar.innerHTML = `
            <div class='electros-titlebar-buttons ${buttonsAlignClass}'>
                ${options.minimizeOnly ? `
                    <button class='electros-titlebar-button' id='minimize-button'></button>
                    <button class='electros-titlebar-button' id='hide-button'></button>
                ` : `
                    <button class='electros-titlebar-button' id='close-button'></button>
                    <button class='electros-titlebar-button' id='minimize-button'></button>
                    <button class='electros-titlebar-button' id='maximize-button'></button>
                    <button class='electros-titlebar-button' id='fullscreen-button'></button>
                `}
            </div>
        `;
    } else {
        titlebar.innerHTML = `
            <div class='electros-titlebar-buttons ${buttonsAlignClass}'>
                <button class='electros-titlebar-button' id='minimize-button'></button>
                ${!options.minimizeOnly ? `
                    <button class='electros-titlebar-button' id='maximize-button'></button>
                    <button class='electros-titlebar-button' id='close-button'></button>
                ` : ''}
            </div>
        `;
    }

    const buttonActions = {
        'close-button': 'close-window',
        'hide-button': 'hide-window',
        'minimize-button': 'minimize-window',
        'maximize-button': 'toggle-full-screen',
        'fullscreen-button': 'maximize-window',
    };

    const buttons = titlebar.querySelector('.electros-titlebar-buttons');
    if (buttons) {
        buttons.setAttribute('data-tauri-drag-region', 'false');
    }

    Object.entries(buttonActions).forEach(([buttonId, action]) => {
        const button = document.getElementById(buttonId);
        if (button) {
            button.addEventListener('click', () => {
                window.electron.invoke(action);
            });
        }
    });
}
