package visibility

type Visibility struct {
	ExportedField   string
	unexportedField string
}

func NewVisibility() *Visibility {
	return &Visibility{
		ExportedField:   "exported",
		unexportedField: "unexported",
	}
}

func (v *Visibility) GetExported() string {
	return v.ExportedField
}

func (v *Visibility) getUnexported() string {
	return v.unexportedField
}

func (v *Visibility) Describe() string {
	return v.GetExported() + v.getUnexported()
}
