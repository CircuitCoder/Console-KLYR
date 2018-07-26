import tmpl from './NewPost.html';
import { request } from '../util';

export default Vue.component('NewPost', {
  template: tmpl,

  data: () => ({
    title: '',
    content: '',
    category: '',
  }),

  methods: {
    async submit() {
      const resp = await request('/api/posts', 'POST', {
        title: this.title,
        author: 'root',
        content: this.content,
        tags: [this.category],
        time: 0, // This is ignored
      });

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Success',
        });
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },
  },
});
