const url            = document.querySelector('[name="site_url"]').value;
const spinner        = document.querySelector('.lds-dual-ring ');
const regexForm      = document.querySelector('[name="items_regex"]');
const autofillButton = document.querySelector('#autofill');

autofillButton.addEventListener('click', async () => {
    autofillButton.style.backgroundColor = '#ffb317';
    spinner.style.display = 'inline-block';

    await fetch(`/api/autofill?url=${encodeURIComponent(url)}`).then(async (resp) => {
        const text = await resp.text();
        console.log(text);
        regexForm.value = text;
        spinner.style.display = 'none';
        autofillButton.style.backgroundColor = '#d5def7';
    });
});
