export default class SecretService {

  createSecretRequest(secretValue, password) {
    return (async () => {

      const response = await fetch('http://localhost:8000/secrets', {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Accept": "*/*",
          "Accept-Encoding": "gzip, deflate, br"
        },
        redirect: "follow",
        referrerPolicy: "no-referrer",
        body: JSON.stringify({
          secret_content: secretValue,
          password: password
        }),
      });
      const content = await response.json();
      return content;
    })();
  }

  getSecretRequest(secretId, password) {
    return (async () => {
      const response = await fetch('http://localhost:8000/secrets/' + secretId + '?' + new URLSearchParams({
            password: password,
        }), {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
        mode: 'no-cors',
        redirect: "follow",
        referrerPolicy: "no-referrer",
      });
      const content = await response.json();
      return content;
    })();
  }
}
