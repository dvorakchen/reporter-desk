export type AlertMsg = {
  id: number;
  title: string;
  desc: string;
};

export const alerts = $state([] as AlertMsg[]);

let id = 0;
const DISPLAY_DURATION = 10_000;

export function pushAlert(title: string, desc: string = "") {
  let alert = {
    id: ++id,
    title,
    desc,
  };

  alerts.push(alert);

  setTimeout(() => {
    alerts.shift();
  }, DISPLAY_DURATION);
}
