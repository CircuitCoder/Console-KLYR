export async function request(url, method = 'GET', payload = null) {
  let body;

  if(method === 'GET' || method === 'HEAD') {
    body = undefined;
  } else {
    body = JSON.stringify(payload);
  }

  const resp = await fetch(url, {
    headers: {
      'Content-Type': 'application/json; charset=utf-8',
    },
    credentials: 'same-origin',
    method,
    body,
  });

  const data = await resp.json();
  return data;
}

export function ping() {
  return 'pong';
}
