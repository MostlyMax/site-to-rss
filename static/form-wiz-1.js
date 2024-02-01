const url            = document.querySelector('[name="site_url"]').value;
const spinner        = document.querySelector('.lds-dual-ring ');
const regexForm      = document.querySelector('[name="items_regex"]');
const autofillButton = document.querySelector('#autofill');
const autofillError  = document.querySelector('.autofill .error');

autofillButton.addEventListener('click', async () => {
    autofillButton.style.backgroundColor = '#ffb317';
    spinner.style.display = 'inline-block';

    await fetch(`/api/autofill?url=${encodeURIComponent(url)}`).then(async (resp) => {
        if (resp.ok) {
            autofillError.style.display = 'none';
            const text = await resp.text();
            regexForm.value = text;
        // we return NotAccepted for when the AI response isn't valid regex
        // "doesn't find any content that conforms to the criteria given by the user agent - Mozilla"
        } else if (resp.status == 406) {
            autofillError.style.display = 'block';
            const text = await resp.text();
            regexForm.value = text;
        } else {
            autofillError.style.display = 'block';
        }
        spinner.style.display = 'none';
        autofillButton.style.backgroundColor = '#d5def7';
    });

});
