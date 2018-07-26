import tmpl from './Chronometer.html';
import { request } from '../util';

export default Vue.component('Chronometer', {
  template: tmpl,
  props: ['chrono', 'chronoDesc'],

  data: () => ({
    ratio: null,

    presets: false,
  }),

  methods: {
    selectPreset(data) {
      if(data.index === 0) this.ratio = '6';
      if(data.index === 1) this.ratio = '1';
      if(data.index === 2) this.ratio = '0';
    },

    async updateRatio() {
      const ratio = parseFloat(this.ratioModel);
      const resp = await request('/api/chrono', 'PUT', {
        ratio,
      });

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Success',
        });

        this.$root.updateChrono();
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },
  },

  computed: {
    ratioModel: {
      get() {
        if(this.ratio === null) {
          return this.chronoDesc ? this.chronoDesc.ratio : 1;
        }

        return this.ratio;
      },
      set(s) {
        this.ratio = s;
      },
    },

    validRatio() {
      if(!this.ratio) return true;
      if(typeof this.ratio === 'number') return true;
      return this.ratio.match(/^((\d+)|(\d*\.\d+))$/) !== null;
    },
  },
});
