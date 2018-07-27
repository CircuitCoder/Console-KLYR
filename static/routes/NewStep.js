import tmpl from './NewStep.html';
import { request } from '../util';

export default Vue.component('NewStep', {
  template: tmpl,

  data: () => ({
    title: '',

    mde: null,
  }),

  async mounted() {
    /* global SimpleMDE */
    this.mde = new SimpleMDE({ element: this.$refs.content });
  },

  methods: {
    async submit() {
      const resp = await request('/api/steps', 'POST', {
        parent: null,

        title: this.title,
        content: this.mde.value(),
        time: 0, // Ignored
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
