const url            = document.querySelector('[name="site_url"]').value;
const spinner        = document.querySelector('.lds-dual-ring ');
const regexForm      = document.querySelector('[name="items_regex"]');
const autofillButton = document.querySelector('#autofill');
const autofillError  = document.querySelector('.autofill .error');

autofillButton.addEventListener('click', async () => {
    autofillButton.style.backgroundColor = '#ffb317';
    spinner.style.display = 'inline-block';

    await fetch(`/api/autofill?url=${encodeURIComponent(url)}`).then(async (resp) => {
        console.log(resp.ok);
        if (resp.ok) {
            autofillError.style.display = 'none';
            const text = await resp.text();
            regexForm.value = text;
        } else {
            console.log('ping');
            autofillError.style.display = 'block';
        }
        spinner.style.display = 'none';
        autofillButton.style.backgroundColor = '#d5def7';
    });

});
