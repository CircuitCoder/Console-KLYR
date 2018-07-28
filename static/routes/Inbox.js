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

    toPost(p) {
      const evt = Object.keys(p.content)[0];
      this.$router.push({
        name: 'Post',
        params: { id: p.content[evt].id },
        query: { pending: evt !== 'ReviewPassed' },
      });
    },

    toStep(p) {
      const evt = Object.keys(p.content)[0];
      this.$router.push({
        name: 'Step',
        params: { id: p.content[evt].id },
      });
    },

    formatTime(t) {
      /* global moment */
      return moment(t * 1000).format('MM/DD HH:mm:ss');
    },
  },
});
