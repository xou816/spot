all: install build

build: doc.tex
	latexmk -c doc.tex

watch: doc.tex
	latexmk -pvc doc.tex

DEPENDENCIES = luatex $\
	fontspec $\
	lm $\
	xcolor $\
	xcolor-solarized $\
	fontawesome $\
	xifthen $\
	ifmtarg $\
	pgf $\
	pgf-blur $\
	ec $\
	etoolbox $\
	xkeyval $\
	minted kvoptions fancyvrb fvextra upquote float ifplatform pdftexcmds xstring lineno framed catchfile

install:
	tlmgr update --self
	tlmgr update texlive-scripts
	tlmgr install $(DEPENDENCIES)

clean:
	latexmk -C