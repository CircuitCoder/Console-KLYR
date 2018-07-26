import tmpl from './Inbox.html';
import { request } from '../util';

export default Vue.component('Inbox', {
  template: tmpl,
  props: ['msgs'],

  methods: {
    async done(m) {
      const resp = await request('/api/msg/done', 'POST', m);

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Success',
        });
        this.$root.updateMsg();
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },
  }
});
